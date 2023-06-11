//! Buddy Page Allocator
//! https://en.wikipedia.org/wiki/Buddy_memory_allocation
//!
//!   NOTE: The allocator is NOT thread-safe.

#[cfg(feature = "module_tests")]
pub mod test;

use crate::arch;
use crate::arch::bits;
use crate::peripherals::memory;
use core::slice;

/// Support blocks that are up to Page Size * 2^10 bytes. For example, with a
/// 4 KiB page size, the largest block size is 4 MiB.
const PAGE_LEVELS: usize = 11;

/// The size of a word in the flags array.
const WORD_SIZE: usize = u8::BITS as usize;

/// Given a block number, shift right by INDEX_SHIFT to get the index into the
/// flags array.
const INDEX_SHIFT: usize = bits::floor_log2(WORD_SIZE);

/// Given a block number, used INDEX_MASK to get the bit number within the flags
/// array word.
const INDEX_MASK: usize = WORD_SIZE - 1;

/// Metadata for each level in the buddy allocator.
#[derive(Default)]
struct PageLevel {
  offset: usize,
  valid: usize,
  avail: usize,
}

/// The Buddy Allocator. The "textbook" implementation uses linked lists of
/// available blocks. Splitting "buddies" involves removing a block from the
/// list at level N and adding two blocks to the list at level N - 1. To keep
/// the allocator as compact as possible, this Buddy Allocator only uses bit
/// flags and an available count. Every bit starts as 1, and allocation sets the
/// appropriate bits at every level to 0 when allocating a block. This is
/// slower, but keeps the allocator much smaller.
///
/// For example, with 512 GiB of physical memory, an allocator needs 1 MiB to
/// represent all possible 64 KiB pages as a single bit at level 0 and a total
/// of ~2 MiB for all levels together.
///
/// A linked list version with a page number and next pointer would require
/// 16 bytes per page meaning 128 MiB would have to be reserved to create nodes
/// for all 64 KiB pages.
pub struct PageAllocator<'memory> {
  page_size: usize,
  page_shift: usize,
  base: usize,
  size: usize,
  flags: &'memory mut [u8],
  levels: [PageLevel; PAGE_LEVELS],
}

impl<'memory> PageAllocator<'memory> {
  /// Calculates the size of the allocator metadata for the given page size and
  /// memory block size.
  ///
  /// # Parameters
  ///
  /// * `page_size` - The page size in bytes.
  /// * `block_size` - The size of the memory block served.
  ///
  /// # Returns
  ///
  /// The allocator metadata size in bytes.
  pub fn calc_size(page_size: usize, block_size: usize) -> usize {
    let (_, size) = PageAllocator::make_levels(page_size, block_size);
    size
  }

  /// Construct a new page allocator.
  ///
  /// # Parameters
  ///
  /// * `page_size` - The page size in bytes.
  /// * `base` - The base address of the memory block served. The base address
  ///   must be on a page boundary.
  /// * `size` - The size of the memory block served.
  /// * `mem` - The memory to use for the allocator struct.
  /// * `excl` - Memory blocks to exclude from the allocator, e.g. the kernel
  ///   area.
  ///
  /// # Description
  ///
  /// `calc_size` should have been called to ensure that `mem` has sufficient
  /// space for the allocator's metadata. In addition to the provided exclusion
  /// ranges, the allocator will exclude its own metadata.
  ///
  /// # Returns
  ///
  /// The allocator structure.
  pub fn new(
    page_size: usize,
    base: usize,
    size: usize,
    mem: *mut u8,
    excl: &memory::MemoryConfig,
  ) -> Self {
    assert!(bits::is_power_of_2(page_size));
    assert!(bits::align_down(base, page_size) == base);

    let size = bits::align_down(size, page_size);
    let (levels, alloc_size) = PageAllocator::make_levels(page_size, size);
    let mut allocator = PageAllocator {
      page_size,
      page_shift: bits::floor_log2(page_size),
      base,
      size,
      flags: unsafe { slice::from_raw_parts_mut(mem, alloc_size) },
      levels,
    };

    // Initialize the metadata.
    allocator.init_flags();

    // Reserve the provided exclusion ranges if they are in the area served by
    // this allocator.
    for r in excl.get_ranges() {
      _ = allocator.reserve(r.base, r.size);
    }

    // Reserve the allocator's own metadata memory.
    let mem_addr = (mem as usize) - arch::get_kernel_virtual_base();
    _ = allocator.reserve(mem_addr, alloc_size);

    allocator
  }

  /// Allocate a physically contiguous block of pages.
  ///
  /// # Parameters
  ///
  /// * `pages` - The number of pages to allocate.
  ///
  /// # Returns
  ///
  /// Ok with the starting physical address of the block if a contiguous block
  /// is found. None if a large enough contiguous block cannot be found or the
  /// requested page count exceeds 2^(PAGE_LEVELS - 1).
  pub fn allocate(&mut self, pages: usize) -> Option<usize> {
    if pages == 0 {
      return None;
    }

    // Calculate the level with the ideal block size.
    let level_idx = bits::ceil_log2(pages);

    // Find the smallest available block. Handles the case where the number of
    // pages requested is too large.
    if let Some((idx, bit)) = self.find_available_block(level_idx) {
      // Allocate the block by splitting as necessary.
      return Some(self.allocate_block(level_idx, idx, bit));
    }

    // Sorry, try another allocator or do some swapping.
    None
  }

  pub fn free(&mut self, base: usize, size: usize) {}

  /// Initialize memory block metadata.
  ///
  /// # Description
  ///
  /// Marks all valid blocks as available.
  fn init_flags(&mut self) {
    self.flags.fill(u8::MAX);

    for level in &mut self.levels {
      let last = level.valid >> INDEX_SHIFT;
      let rem = level.valid - (last << INDEX_SHIFT);
      self.flags[level.offset + last] = (1 << rem) - 1;
      level.avail = level.valid;
    }
  }

  /// Reserve a block of memory.
  ///
  /// # Description
  ///
  ///   TODO: Setting individual bits is probably a very inefficient way to do
  ///         this. However, reservations are a one-time setup thing, so for now
  ///         this is probably fine.
  ///
  /// # Parameters
  ///
  /// * `base` - The address of the block to reserve.
  /// * `size` - The size of the block to reserve.
  ///
  /// # Returns
  ///
  /// True if the block resides completely within the area served by this
  /// allocator and the size is greater than zero, false otherwise.
  fn reserve(&mut self, base: usize, size: usize) -> bool {
    if size == 0 {
      return false;
    }

    if base < self.base {
      return false;
    }

    let last = bits::align_up(base + size, self.page_size) - 1;

    if last >= self.base + self.size {
      return false;
    }

    let base = bits::align_down(base, self.page_size);

    // Get the starting and ending pages at level 0.
    let mut start = (base - self.base) >> self.page_shift;
    let mut end = (last - self.base) >> self.page_shift;

    for level in self.levels.iter_mut() {
      let mut idx = level.offset + (start >> INDEX_SHIFT);
      let mut mask = 1 << (start & INDEX_MASK);

      for _ in start..=end {
        // Only subtract from the available count if this block has not already
        // been reserved. This keeps the accounting correct if two reserved
        // blocks overlap.
        //
        //   NOTE: True is guaranteed to be 1 and false is guaranteed to be 0.
        level.avail -= ((self.flags[idx] & mask) != 0) as usize;

        // Reserve the block.
        self.flags[idx] &= !mask;

        mask <<= 1;

        if mask == 0 {
          idx += 1;
          mask = 1;
        }
      }

      // Moving up to the next level, shift the starting and ending blocks down
      // by one, effectively dividing by two as the block sizes double.
      start >>= 1;
      end >>= 1;
    }

    true
  }

  /// Calculates the size of the allocator metadata for the given page size and
  /// memory block size, and constructs a list of descriptors for the allocator
  /// levels.
  ///
  /// # Parameters
  ///
  /// * `page_size` - The page size in bytes.
  /// * `block_size` - The size of the memory block served.
  ///
  /// # Returns
  ///
  /// The a list of descriptors and the allocator metadata size in bytes.
  ///
  /// # Assumes
  ///
  /// Assumes that the page size has already been validated for the
  /// architecture.
  fn make_levels(page_size: usize, block_size: usize) -> ([PageLevel; PAGE_LEVELS], usize) {
    let mut levels: [PageLevel; PAGE_LEVELS] = Default::default();
    let mut blocks = block_size / page_size;
    let mut offset = 0;

    for i in 0..PAGE_LEVELS {
      levels[i] = PageLevel {
        offset,
        valid: blocks,
        avail: 0,
      };

      offset += (blocks + WORD_SIZE - 1) >> INDEX_SHIFT;
      blocks >>= 1;
    }

    (levels, offset)
  }

  /// Finds the first available block.
  ///
  /// # Parameters
  ///
  /// * `level_idx` - The level to search.
  ///
  /// # Returns
  ///
  /// A tuple with the word index of the available block relative to the start
  /// of the flags for `level_idx` and a mask specifying the bit of the
  /// available block within the word.
  fn find_available_block(&self, level_idx: usize) -> Option<(usize, u8)> {
    // Requested block size is too large.
    if level_idx >= PAGE_LEVELS {
      return None;
    }

    let level = &self.levels[level_idx];

    // No available blocks.
    if level.avail == 0 {
      return None;
    }

    let start = level.offset;
    let end = start + (level.valid >> INDEX_SHIFT);

    for idx in start..=end {
      let word = self.flags[idx];
      let bit = bits::least_significant_bit(word as usize) as u8;

      if bit != 0 {
        return Some((idx - level.offset, bit));
      }
    }

    // No free block found, but the available block count is greater than zero.
    debug_assert!(false, "Free block accounting is incorrect.");
    None
  }

  /// Allocate a block.
  ///
  /// # Parameters
  ///
  /// * `level_idx` - The level with an available block.
  /// * `idx` - The word index of the available block. The index is relative to
  ///   the start of the level's flags, not the start of the metadata.
  /// * `bit` - The bit mask of the available block.
  ///
  /// # Description
  ///
  /// Consider a free block at level 2, index 2, bit 0b100.
  /// `log2( 0b100 ) = 2`, so the block of interest is the third block in the
  /// third word at level 2 and its offset is `( 2 << 3 ) + 2 = 18`. The page
  /// offset is `18 << 2 = 72`, so the starting address is
  /// `base + ( 72 * page size )`. If, for example the page size is 4 KiB and
  /// the base address is 0, the starting address is 0x48000.
  ///
  /// At level 2, a block is `1 << 2 = 4` pages long. So, the size is simply
  /// `( 1 << 2 ) * page size`. Again, if the page size is 4 KiB, then the block
  /// is 16 KiB long.
  ///
  /// Knowing the base address and the size, allocation is no different than
  /// reservation.
  ///
  /// # Returns
  ///
  /// The base address of the allocated block.
  fn allocate_block(&mut self, level_idx: usize, idx: usize, mask: u8) -> usize {
    // Compute the base address of the block and its size.
    let block = (idx << INDEX_SHIFT) + bits::floor_log2(mask as usize);
    let addr = self.base + ((block << level_idx) * self.page_size);
    let size = (1 << level_idx) * self.page_size;

    // Now simply reserve the block.
    self.reserve(addr, size);

    addr
  }
}
