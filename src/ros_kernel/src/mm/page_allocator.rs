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
use crate::peripherals::memory;
use crate::support::bits;
use core::{cmp, ptr, slice};

/// Support blocks that are up to Page Size * 2^10 bytes. For example, with a
/// 4 KiB page size, the largest block size is 4 MiB.
const BLOCK_LEVELS: usize = 11;

/// Bit length of metadata word.
const WORD_BITS: usize = usize::BITS as usize;

/// Word byte-size shift.
const WORD_SHIFT: usize = bits::floor_log2(WORD_BITS >> 3);

/// Masks a block number to find a block's index within a word.
const WORD_MASK: usize = WORD_BITS - 1;

/// Shift count for the metadata index of a block's word.
const INDEX_SHIFT: usize = bits::floor_log2(WORD_BITS);

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
    bits::xor_checksum(&[next, prev])
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

  /// Construct the block level metadata for an allocator.
  ///
  /// # Parameters
  ///
  /// * `size` - The size of the memory area served by the allocator.
  ///
  /// # Returns
  ///
  /// A tuple with the block level metadata and the required size of the flag
  /// metadata in bytes.
  fn make_levels(size: usize) -> ([BlockLevel; BLOCK_LEVELS], usize) {
    let page_shift = arch::get_page_shift();

    let mut levels: [BlockLevel; BLOCK_LEVELS] = Default::default();
    let mut blocks = size >> page_shift;
    let mut offset = 0;

    for level in &mut levels {
      level.offset = offset;

      // One bit per pair of blocks.
      let bits = (blocks + 1) >> 1;

      // Round up the number of bits to whole words.
      offset += (bits + WORD_BITS - 1) >> INDEX_SHIFT;

      blocks >>= 1;
    }

    (levels, offset << WORD_SHIFT)
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
  /// Verifies that the pointer is page-aligned and that the node's checksum is
  /// correct.
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

  /// Construct a new page allocator for a given contiguous memory area.
  ///
  /// # Parameters
  ///
  /// * `base` - Base physical address of the memory area served.
  /// * `size` - Size of the memory area.
  /// * `mem` - Pointer to a memory block available for metadata.
  /// * `avail` - Available physical regions with the memory area.
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
  /// A new allocator, or None if:
  ///
  /// * `base` is 0 after alignment.
  /// * `size` is less than the page size after alignment.
  /// * `base + size` would overflow a pointer after alignment.
  /// * `mem` is null.
  /// * `avail` is empty.
  pub fn new(base: usize, size: usize, mem: *mut u8, avail: &memory::MemoryConfig) -> Option<Self> {
    let page_size = arch::get_page_size();
    let virtual_base = arch::get_kernel_virtual_base();

    // Align the base and size down.
    let base = bits::align_down(base, page_size);
    let size = bits::align_down(size, page_size);

    if base == 0 {
      return None;
    }

    if size < page_size {
      return None;
    }

    // Ensure the base address is physical.
    if base & virtual_base != 0 {
      return None;
    }

    // Ensure that the size is not going to overflow a pointer. Note that we're
    // use the virtual base address rather than usize::MAX.
    if virtual_base - base < size {
      return None;
    }

    if mem == ptr::null_mut() {
      return None;
    }

    if avail.is_empty() {
      return None;
    }

    let (levels, alloc_size) = Self::make_levels(size);

    let mut allocator = PageAllocator {
      base: virtual_base + base,
      size,
      levels,
      flags: unsafe { slice::from_raw_parts_mut(mem as *mut usize, alloc_size >> WORD_SHIFT) },
    };

    allocator.init_metadata(&avail);

    Some(allocator)
  }

  /// Attempts to allocate a contiguous block of pages.
  ///
  /// # Parameters
  ///
  /// * `pages` - The requested number of pages.
  ///
  /// # Description
  ///
  /// If `pages` is not a power of 2, the size of the block returned will be the
  /// smallest power of 2 pages larger than the requested number of pages.
  ///
  /// # Returns
  ///
  /// A tuple with the base physical address of the contigous block and the
  /// actual number of pages allocated, or None if the allocator could not find
  /// an available contigous block of the requested size.
  pub fn allocate(&mut self, pages: usize) -> Option<(usize, usize)> {
    if pages == 0 {
      return None;
    }

    // Calculate the level with the minimum block size.
    let min_level = bits::ceil_log2(pages);

    for level in min_level..BLOCK_LEVELS {
      if self.levels[level].head == 0 {
        continue;
      }

      let virtual_base = arch::get_kernel_virtual_base();
      let block = self.split_free_block(level, min_level);
      let pages = 1 << min_level;
      return Some((block - virtual_base, pages));
    }

    // No blocks available.
    None
  }

  /// Frees a block of memory.
  ///
  /// # Parameters
  ///
  /// * `base` - The base physical address of the block.
  /// * `pages` - The number of pages in the block.
  ///
  /// # Description
  ///
  /// The number of pages must be a power of 2. The base address of the block
  /// must be aligned on an address that is a multiple of the block size. The
  /// function ignores a base address of 0 or a page count of 0.
  pub fn free(&mut self, base: usize, pages: usize) {
    if (base == 0) || (pages == 0) {
      return;
    }

    assert!(bits::is_power_of_2(pages));

    let min_level = bits::floor_log2(pages);
    assert!(min_level < BLOCK_LEVELS);
    assert!(base & (pages - 1) == 0);

    let page_shift = arch::get_page_shift();
    let virtual_base = arch::get_kernel_virtual_base();
    let mut base_addr = base + virtual_base;

    for level in min_level..BLOCK_LEVELS {
      let (index, bit_idx) = self.get_flag_index_and_bit(base_addr, level);

      // The allocator does not protect against double-free, so the assumption
      // here is that the buddy block is in use if the bit is zero and we cannot
      // coalesce the two.
      if self.flags[index] & (1 << bit_idx) == 0 {
        self.add_to_list(level, base_addr);
        break;
      }

      // If the bit is not zero, get the buddy block address using XOR. Remove
      // the buddy from the list at this level, then update the base address to
      // the minimum of the two.
      let buddy_addr = base_addr ^ ((1 << level) << page_shift);
      self.remove_from_list(level, buddy_addr);
      base_addr = cmp::min(base_addr, buddy_addr);
    }
  }

  /// Initializes the allocator's linked list and accounting meta data.
  ///
  /// # Parameters
  ///
  /// * `avail` - Available physical regions with the memory area.
  fn init_metadata(&mut self, avail: &memory::MemoryConfig) {
    let page_shift = arch::get_page_shift();
    let page_size = arch::get_page_size();
    let virtual_base = arch::get_kernel_virtual_base();

    self.flags.fill(0);

    for range in avail.get_ranges() {
      // If the range has an invalid base, skip it.
      if range.base & virtual_base != 0 {
        continue;
      }

      // If the range has an invalid size, skip it.
      if virtual_base - range.base < range.size {
        continue;
      }

      // We now know that adding the size to the base is safe. We know from the
      // checks in Self::new() that adding the memory area size to the memory
      // area base is safe. Now check if the range is fully enclosed within the
      // memory area. If not, skip it.
      let end = range.base + range.size;
      let mut addr = range.base;
      let mut remaining = range.size;

      if (addr + virtual_base) < self.base || (end + virtual_base) > (self.base + self.size) {
        continue;
      }

      while remaining >= page_size {
        // Consider the address 0x1ed000. With 4 KiB pages, this address is
        // 0x1ed pages from the beginning of the address space. Each block must
        // be exactly aligned on a multiple of its size. We can figure out the
        // alignment using the least-significant 1 bit in the block number. For
        // example, 0x1ed = 0b111101101. The least-significant 1 bit is bit 0,
        // so the address is aligned on a 1-page multiple and we cannot allocate
        // more than a single page at that address.
        //
        // After making a single page block available at 0x1ed000, we increment
        // the address to 0x1ee000. This is block 0x1ee = 0b111101110. This
        // address is aligned on a 2-page multiple. So, we make a 2-page block
        // available and increment the address to 0x1f0000. This address is
        // aligned on a 16-page multiple, so the next address is 0x200000. This
        // address is aligned on a 512-page multiple, and so on.
        //
        // Page 0 should never be used.
        let page_num = addr >> page_shift;
        let addr_align = bits::least_significant_bit(page_num);
        let max_level = cmp::min(bits::floor_log2(addr_align), BLOCK_LEVELS - 1);

        // Of course, the above is only half the story. We also have to cap the
        // maximum block size by the remaining memory size.
        let pages_remaining = remaining >> page_shift;
        let level = cmp::min(bits::floor_log2(pages_remaining), max_level);
        let blocks = 1 << level;
        let size = blocks << page_shift;

        // Add the block to the level's available list.
        self.add_to_list(level, addr + virtual_base);

        addr += size;
        remaining -= size;
      }
    }
  }

  /// Get the flag index and bit for a given virtual address at a given level.
  ///
  /// # Parameters
  ///
  /// * `block_addr` - The virtual block address.
  /// * `level` - The block level.
  ///
  /// # Description
  ///
  /// Assumes that the start address for the block is aligned on a multiple of
  /// the block size for the specified level.
  ///
  /// # Returns
  ///
  /// A tuple with the absolute word index into the metadata flags and the bit
  /// index in that word for the block.
  fn get_flag_index_and_bit(&self, block_addr: usize, level: usize) -> (usize, usize) {
    let page_shift = arch::get_page_shift();
    let page_num = (block_addr - self.base) >> page_shift;
    let block_num = page_num >> level;
    let block_pair = block_num >> 1;
    let index = self.levels[level].offset + (block_pair >> INDEX_SHIFT);
    let bit = block_pair & WORD_MASK;
    
    (index, bit)
  }

  /// Split a free block until it is the required size.
  ///
  /// # Parameters
  ///
  /// * `level` - The level at which to split.
  /// * `min_level` - The level at which the split stops.
  ///
  /// # Description
  ///
  /// Assumes at least one block is available at `level`. Removes the first
  /// available block, splits it in half, and adds the odd half to the first
  /// list at `level - 1`. Repeats until reaching `min_level`.
  ///
  /// # Returns
  ///
  /// The block address of the block removed from `level`.
  fn split_free_block(&mut self, level: usize, min_level: usize) -> usize {
    let page_size = arch::get_page_size();
    let block_addr = self.pop_from_list(level);

    // For this example, just assume 1 byte pages starting at 0 for simplicity.
    //
    // Assume block 2 is free at level 4 covering pages [32, 48), and assume we
    // want to allocate two pages. Remove 0x20 from block 4. At level 3, the odd
    // buddy is 0x20 | 0x08:
    //
    //  0x20                             0x28                             0x30
    //   +--------+--------+----------------+--------------------------------+
    //   |                                  |                                |
    //   +--------+--------+----------------+--------------------------------+
    //
    // Add 0x28 to the free list at level 3 to cover pages [40, 48), then move
    // down. At level 2, the odd buddy is 0x20 | 0x04:
    //
    //  0x20            0x24             0x28
    //   +--------+--------+----------------+----
    //   |                 |                |
    //   +--------+--------+----------------+----
    //
    // Add 0x24 to the free list at level 2 to cover pages [36, 40), then move
    // down. At level 1, the odd buddy is 0x20 | 0x02:
    //
    //  0x20   0x22     0x24
    //   +--------+--------+----
    //   |        |        |
    //   +--------+--------+----
    //
    // Add 0x22 to the free list at level 1 to cover pages [34, 36). We are now
    // done splitting and can return 0x20 as the two-page block covering pages
    // [32, 34).
    for l in (min_level..level).rev() {
      let buddy_addr = block_addr | (page_size << l);
      self.add_to_list(l, buddy_addr);
    }

    block_addr
  }

  /// Adds a block to the tail of a level's list of available blocks.
  ///
  /// # Parameters
  ///
  /// * `level` - The level to which the block will be added.
  /// * `block_addr` - The virtual block address to add to the list.
  fn add_to_list(&mut self, level: usize, block_addr: usize) {
    let (index, bit_idx) = self.get_flag_index_and_bit(block_addr, level);
    let head_addr = self.levels[level].head;
    let block = Self::get_block_node_unchecked_mut(block_addr);

    // If the list is empty, initialize a new node that points only to itself
    // and return the block address as the new head address. Otherwise, add the
    // block to the tail of the list.
    if head_addr == 0 {
      *block = BlockNode::new(block_addr, block_addr);
      self.levels[level].head = block_addr;
    } else {
      let head = Self::get_block_node_mut(head_addr);
      let prev = Self::get_block_node_mut(head.prev);

      *block = BlockNode::new(head_addr, head.prev);
      *head = BlockNode::new(head.next, block_addr);
      *prev = BlockNode::new(block_addr, prev.prev);
    }

    self.flags[index] ^= 1 << bit_idx;
  }

  /// Pop the head of a level's free list.
  ///
  /// # Parameters
  ///
  /// * `level` - The level from which to remove a free block.
  ///
  /// # Description
  ///
  /// Assumes that the list is not empty.
  ///
  /// # Returns
  ///
  /// The block address popped from the list.
  fn pop_from_list(&mut self, level: usize) -> usize {
    let head_addr = self.levels[level].head;
    self.remove_from_list(level, head_addr);

    head_addr
  }

  /// Removes a specific block from a level's free list.
  ///
  /// # Parameters
  ///
  /// * `level` - The level from which to remove a free block.
  /// * `block_addr` - The virtual block address to remove from the list.
  fn remove_from_list(&mut self, level: usize, block_addr: usize) {
    let (index, bit_idx) = self.get_flag_index_and_bit(block_addr, level);
    let head_addr = self.levels[level].head;
    let block = Self::get_block_node(block_addr);

    // If the block address is the same as the head address, then we need to
    // check if the head points to itself. If it does, simply set the list head
    // to zero. Otherwise, we assume there is more than one block and perform a
    // normal list removal.
    if block_addr == head_addr {
      let head = Self::get_block_node(head_addr);

      if head.next == head_addr {
        self.levels[level].head = 0;
      }
    } else {
      let prev = Self::get_block_node_mut(block.prev);
      let next = Self::get_block_node_mut(block.next);

      *prev = BlockNode::new(prev.prev, block.next);
      *next = BlockNode::new(block.prev, next.next);
    }

    self.flags[index] ^= 1 << bit_idx;
  }
}
