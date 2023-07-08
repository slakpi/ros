//! Buddy Page Allocator
//!
//! https://en.wikipedia.org/wiki/Buddy_memory_allocation
//! https://www.kernel.org/doc/gorman/html/understand/understand009.html
//!
//!   NOTE: The allocator is NOT thread-safe.
//!   NOTE: The allocator does NOT protect against double-free bugs/attacks.

#[cfg(feature = "module_tests")]
pub mod test;

use crate::arch;
use crate::arch::bits;
use crate::peripherals::memory;
use core::{cmp, mem, slice};

/// Support blocks that are up to Page Size * 2^10 bytes. For example, with a
/// 4 KiB page size, the largest block size is 4 MiB.
const BLOCK_LEVELS: usize = 11;

/// Bit length of metadata word.
const WORD_BITS: usize = usize::BITS as usize;

/// Word byte-size shift.
const WORD_SHIFT: usize = bits::floor_log2(mem::size_of::<usize>());

/// Masks a block number to find a block's index within a word.
const WORD_MASK: usize = WORD_BITS - 1;

/// Shift count for the metadata index of a block's word.
const INDEX_SHIFT: usize = bits::floor_log2(WORD_BITS);

/// Initial value for the simple XOR checksum.
const CHECKSUM_SEED: usize = 0xbaadf00d;

/// Linked-list node placed at the beginning of each unallocated block.
#[repr(C)]
struct BlockNode {
  next: usize,
  prev: usize,
  checksum: usize,
}

impl BlockNode {
  /// Construct a new block node with a checksum.
  ///
  /// # Parameters
  ///
  /// * `next` - A node's next pointer.
  /// * `prev` - A node's previous pointer.
  ///
  /// # Returns
  ///
  /// A new node.
  fn new(next: usize, prev: usize) -> Self {
    BlockNode {
      next,
      prev,
      checksum: Self::calc_checksum(next, prev),
    }
  }

  /// Calculate a checksum for the given next and previous pointers.
  ///
  /// # Parameters
  ///
  /// * `next` - A node's next pointer.
  /// * `prev` - A node's previous pointer.
  ///
  /// # Description
  ///
  /// The checksum is meant for simple error detection. It is not meant for
  /// error correction or security.
  ///
  /// # Returns
  ///
  /// A checksum.
  fn calc_checksum(next: usize, prev: usize) -> usize {
    (CHECKSUM_SEED ^ next) ^ prev
  }
}

/// Block level metadata
#[derive(Default)]
struct BlockLevel {
  head: usize,
  offset: usize,
}

/// The Buddy Allocator
pub struct PageAllocator<'memory> {
  base: usize,
  size: usize,
  levels: [BlockLevel; BLOCK_LEVELS],
  flags: &'memory mut [usize],
}

impl<'memory> PageAllocator<'memory> {
  /// Calculate the amount of memory required for the allocator's metadata.
  ///
  /// # Parameters
  ///
  /// * `size` - The size of the memory area to be served by the allocator.
  ///
  /// # Returns
  ///
  /// The size of the metadata area in bytes.
  pub fn calc_metadata_size(size: usize) -> usize {
    let (_, size) = Self::make_levels(size);
    size
  }

  /// Construct a new page allocator for a given memory area.
  ///
  /// # Parameters
  ///
  /// * `base` - Base physical address of the memory area served.
  /// * `size` - Size of the memory area.
  /// * `mem` - Pointer to a memory block available for metadata.
  /// * `avail` - Available regions with the memory area.
  ///
  /// # Description
  ///
  /// Assumes that the caller has previously called `calc_metadata_size` and
  /// verified that the memory pointed to by `mem` is large enough.
  ///
  /// The list of available regions should exclude any regions within the memory
  /// area that the allocator should not use. If the memory reserved for the
  /// allocator's metadata is within the memory area, it too should be excluded
  /// from the available regions.
  ///
  /// If the base memory address is not page-aligned, it will be aligned down.
  /// If the size is not page-aligned, it too will be aligned down.
  ///
  /// # Returns
  ///
  /// A new allocator if the parameters are valid.
  pub fn new(base: usize, size: usize, mem: *mut u8, avail: &memory::MemoryConfig) -> Option<Self> {
    let page_size = arch::get_page_size();

    // Align the base and size down.
    let base = bits::align_down(base, page_size);
    let size = bits::align_down(size, page_size);

    // Ensure that the size is not going to overflow a pointer.
    if usize::MAX - base < size {
      return None;
    }

    // Initialize the block levels.
    let (levels, alloc_size) = Self::make_levels(size);

    let mut allocator = PageAllocator {
      base,
      size,
      levels,
      flags: unsafe { slice::from_raw_parts_mut(mem as *mut usize, alloc_size >> WORD_SHIFT) },
    };

    allocator.init_metadata(&avail);

    Some(allocator)
  }

  fn init_metadata(&mut self, avail: &memory::MemoryConfig) {
    let page_shift = arch::get_page_shift();
    let page_size = arch::get_page_size();
    let kernel_base = arch::get_kernel_virtual_base();

    self.flags.fill(0);

    for range in avail.get_ranges() {
      let mut addr = range.base;
      let mut remaining = range.size;

      while remaining >= page_size {
        // Consider the address 0x1ed000. With 4 KiB pages, this address is
        // 0x1ed pages from the beginning of the address space. Each block must
        // be exactly aligned on a multiple of its size. We can figure out the
        // alignment using the least-significant 1 bit in the block number. For
        // example, 0x1ed = 0b111101101. The least-significant 1 bit is bit 0,
        // so the address is aligned on a 1-page multiple and we cannot allocate
        // more than a single page at that address.
        //
        // After make a single page block available at 0x1ed000, we increment
        // the address to 0x1ee000. This is block 0x1ee = 0b111101110. This
        // address is aligned on a 2-page multiple. So, we make a 2-page block
        // available and increment the address to 0x1f0000. This address is
        // aligned on a 16-page block, so the next address is 0x200000 and we
        // can now make a 512-page block available and so on.
        let page_num = (addr - self.base) >> page_shift;
        let addr_align = bits::least_significant_bit(page_num);
        let max_level = if page_num == 0 {
          BLOCK_LEVELS - 1
        } else {
          cmp::min(bits::floor_log2(addr_align), BLOCK_LEVELS - 1)
        };

        // Of course, the above is only half the story. We also have to cap the
        // maximum block size by the remaining memory size.
        let pages_remaining = remaining >> page_shift;
        let level = cmp::min(bits::floor_log2(pages_remaining), max_level);
        let blocks = 1 << level;
        let size = blocks << page_shift;

        // Find the flag array index and bit index for the new block.
        let block_num = page_num >> level;
        let index = self.levels[level].offset + (block_num >> INDEX_SHIFT);
        let bit = block_num & WORD_MASK;

        let virt_addr = addr + kernel_base;

        Self::add_to_list(self.levels[level].head, virt_addr);
        self.levels[level].head = virt_addr;
        self.flags[index] ^= 1 << bit;

        addr += size;
        remaining -= size;
      }
    }
  }

  /// Get a reference to a block's linked-list node.
  ///
  /// # Parameters
  ///
  /// * `addr` - Pointer to the block.
  ///
  /// # Description
  ///
  /// Verifies that the pointer is page-aligned and that the node's checksum is
  /// correct.
  ///
  /// # Returns
  ///
  /// A node reference.
  fn get_block_node(addr: usize) -> &'static BlockNode {
    Self::get_block_node_mut(addr)
  }

  /// Get a mutable reference to a block's linked-list node.
  ///
  /// # Parameters
  ///
  /// * `addr` - Pointer to the block.
  ///
  /// # Description
  ///
  /// Verifies the node's checksum.
  ///
  /// # Returns
  ///
  /// A mutable node reference.
  fn get_block_node_mut(addr: usize) -> &'static mut BlockNode {
    let node = Self::get_block_node_unchecked_mut(addr);
    let checksum = BlockNode::calc_checksum(node.next, node.prev);
    assert!(node.checksum == checksum);

    node
  }

  /// Get a mutable reference to a block's linked-list node.
  ///
  /// # Parameters
  ///
  /// * `addr` - Pointer to the block.
  ///
  /// # Description
  ///
  /// Verifies that the pointer is page-aligned, but does not verify the check-
  /// sum. Used when the node is not expected to be initialized.
  ///
  /// # Returns
  ///
  /// A mutable, uninitialized node reference.
  fn get_block_node_unchecked_mut(addr: usize) -> &'static mut BlockNode {
    let page_size = arch::get_page_size();
    assert!(bits::align_down(addr, page_size) == addr);
    unsafe { &mut *(addr as *mut BlockNode) }
  }

  fn add_to_list(head_addr: usize, block_addr: usize) {
    let block = Self::get_block_node_unchecked_mut(block_addr);

    if head_addr == 0 {
      *block = BlockNode::new(block_addr, block_addr);
      return;
    }

    let head = Self::get_block_node_mut(head_addr);
    let prev = Self::get_block_node_mut(head.prev);

    *block = BlockNode::new(head_addr, head.prev);
    *head = BlockNode::new(head.next, block_addr);
    *prev = BlockNode::new(block_addr, prev.prev);
  }

  fn make_levels(size: usize) -> ([BlockLevel; BLOCK_LEVELS], usize) {
    let page_shift = arch::get_page_shift();

    let mut levels: [BlockLevel; BLOCK_LEVELS] = Default::default();
    let mut blocks = size >> page_shift;
    let mut offset = 0;

    for level in &mut levels {
      level.offset = offset;

      let bits = (blocks + 1) >> 1;
      offset += (bits + WORD_BITS - 1) >> INDEX_SHIFT;
      blocks >>= 1;
    }

    (levels, offset << WORD_SHIFT)
  }
}
