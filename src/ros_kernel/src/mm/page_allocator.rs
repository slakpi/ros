//! Buddy Page Allocator
//! https://en.wikipedia.org/wiki/Buddy_memory_allocation
//!
//!   NOTE: The allocator is NOT thread-safe.

#[cfg(feature = "module_tests")]
pub mod test;

use crate::arch;
use crate::arch::bits;
use crate::peripherals::memory;
use core::{mem, slice};

/// Support blocks that are up to Page Size * 2^10 bytes. For example, with a
/// 4 KiB page size, the largest block size is 4 MiB.
const PAGE_LEVELS: usize = 11;

/// The bit length of a word in the flags array.
const WORD_BITS: usize = usize::BITS as usize;

/// The byte size of a word in the flags array.
const WORD_SIZE: usize = mem::size_of::<usize>();

/// Given a block number, shift right by INDEX_SHIFT to get the index into the
/// flags array.
const INDEX_SHIFT: usize = bits::floor_log2(WORD_BITS);

/// Given a block number, used INDEX_MASK to get the bit number within the flags
/// array word.
const INDEX_MASK: usize = WORD_BITS - 1;

/// Metadata for each level in the buddy allocator.
#[derive(Default)]
struct PageLevel {
  offset: usize,
  valid: usize,
  avail: usize,
}

/// The Buddy Allocator.
///
/// The textbook Buddy Allocator described by Knuth in The Art of Computer
/// Programming, Vol. 1 uses doubly-linked lists for each level to track
/// available blocks. The links are stored along with an available tag and size
/// class at the beginning of the actual block of memory. On a 64-bit system,
/// this requires roughly 17 bytes (1 byte + 8 bytes + 8 bytes) for each block,
/// and those bytes have to be protected.
///
/// This implementation takes a different approach. Different...I have no idea
/// if it is better or worse. I have no idea what I am doing. I am not a kernel
/// developer. Anyhow, this implementation keeps the metadata separate from the
/// blocks themselves in a reserved spot at the end of the memory area served
/// by the allocator. It uses a single bit per block at every level.
///
/// Assuming 8 GiB of physical memory and 4 KiB pages, this results in a fixed
/// metadata size of ~256 KiB. 512 GiB and 64 KiB pages has a fixed metadata
/// size of ~1 MiB. The cost is having to do loops and bit operations at every
/// level rather than simple linked list operations. The allocator tries to
/// offset this by grouping blocks at each level into pointer-sized groups and
/// using relatively fast bit hacks.
pub struct PageAllocator<'memory> {
  base: usize,
  size: usize,
  flags: &'memory mut [usize],
  levels: [PageLevel; PAGE_LEVELS],
}

impl<'memory> PageAllocator<'memory> {
  /// Calculates the size of the allocator metadata for the given page size and
  /// memory block size.
  ///
  /// # Parameters
  ///
  /// * `size` - The size of the memory block served.
  ///
  /// # Returns
  ///
  /// The allocator metadata size in bytes.
  pub fn calc_metadata_size(size: usize) -> usize {
    let (_, size) = PageAllocator::make_levels(size);
    size
  }

  /// Construct a new page allocator.
  ///
  /// # Parameters
  ///
  /// * `base` - The base address of the memory block served. The base address
  ///   must be on a page boundary.
  /// * `size` - The size of the memory block served.
  /// * `mem` - The memory to use for the allocator struct.
  /// * `excl` - Memory blocks to exclude from the allocator, e.g. the kernel
  ///   area.
  ///
  /// # Description
  ///
  /// `calc_metadata_size` should have been called to ensure that `mem` has
  /// sufficient space for the allocator's metadata. In addition to the provided
  /// exclusion ranges, the allocator will exclude its own metadata.
  ///
  /// # Returns
  ///
  /// The allocator structure.
  pub fn new(base: usize, size: usize, mem: *mut u8, excl: &memory::MemoryConfig) -> Option<Self> {
    let page_size = arch::get_page_size();

    // Align the base and size down.
    let base = bits::align_down(base, page_size);
    let size = bits::align_down(size, page_size);

    // Ensure that the size is not going to overflow a pointer.
    if usize::MAX - base < size {
      return None;
    }

    let (levels, alloc_size) = PageAllocator::make_levels(size);

    let mut allocator = PageAllocator {
      base,
      size,
      flags: unsafe { slice::from_raw_parts_mut(mem as *mut usize, alloc_size / WORD_SIZE) },
      levels,
    };

    // Initialize the metadata.
    allocator.init_metadata();

    // Reserve the provided exclusion ranges.
    for r in excl.get_ranges() {
      _ = allocator.reserve(r.base, r.size);
    }

    // Reserve the allocator's own metadata memory.
    let mem_addr = (mem as usize) - arch::get_kernel_virtual_base();
    _ = allocator.reserve(mem_addr, alloc_size);

    Some(allocator)
  }

  /// Allocate a physically contiguous block of pages.
  ///
  /// # Parameters
  ///
  /// * `pages` - The number of pages to allocate.
  ///
  /// # Returns
  ///
  /// Ok with the starting physical address of the block and the actual number
  /// of pages allocated if a contiguous block is found. None if a large enough
  /// contiguous block cannot be found or the requested page count exceeds
  /// 2^(PAGE_LEVELS - 1).
  pub fn allocate(&mut self, pages: usize) -> Option<(usize, usize)> {
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

  pub fn free(&mut self, base: usize, pages: usize) {
    // We can ignore deallocation of zero pages.
    if pages == 0 {
      return;
    }

    let page_size = arch::get_page_size();
    let page_shift = arch::get_page_shift();

    // Allocation can just ignore a request for too much memory. Here we are
    // going to panic if the base address is invalid or the size is out of
    // range.
    let size = pages * page_size;
    assert!(base >= self.base);

    let base = base - self.base;
    assert!(bits::align_down(base, page_size) == base);
    assert!(usize::MAX - base >= size);

    let start = base >> page_shift;
    let end = start + pages - 1;

    // Start by marking all of the pages as available at level 0. This is the
    // reverse of the reservation algorithm applied to level 0 only.
    self.first_level_free(start, end);

    // Now apply a coalescing algorithm at all higher levels.
    self.recursive_free(start, end);
  }

  fn first_level_free(&mut self, start: usize, end: usize) {
    let start_idx = self.levels[0].offset + (start >> INDEX_SHIFT);
    let end_idx = self.levels[0].offset + (end >> INDEX_SHIFT);
    let last_idx = self.levels[0].offset + (self.levels[0].valid >> INDEX_SHIFT);

    let mut mask = usize::MAX << (start & INDEX_MASK);

    // If `start_idx == end_idx`, we'll skip the loop and clear the bits after
    // the end bit. Otherwise, the loop will execute up to, but not including,
    // the end index resetting the mask every time.
    for idx in start_idx..end_idx {
      // If any of the bits are set, we have a double free situation.
      assert!(self.flags[idx] & mask == 0);
      self.levels[0].avail += bits::ones(mask);
      self.flags[idx] |= mask;

      // Reset the mask for the next word.
      mask = usize::MAX;
    }

    let valid_mask = if end_idx == last_idx {
      1usize.wrapping_shl((self.levels[0].valid & INDEX_MASK) as u32) - 1
    } else {
      usize::MAX
    };

    // Intersect the mask with the valid bits in the last word.
    mask &= valid_mask;
    mask &= usize::MAX >> (WORD_BITS - (end & INDEX_MASK) - 1);

    // Perform the last iteration of the loop on the final index.
    assert!(self.flags[end_idx] & mask == 0);
    self.levels[0].avail += bits::ones(mask);
    self.flags[end_idx] |= mask;
  }

  fn recursive_free(&mut self, start: usize, end: usize) {
    let mut start = start >> 1;
    let mut end = end >> 1;
    let half_shift = WORD_BITS >> 1;

    for i in 1..PAGE_LEVELS {
      let start_idx = self.levels[i].offset + (start >> INDEX_SHIFT);
      let end_idx = self.levels[i].offset + (end >> INDEX_SHIFT);
      let last_idx = self.levels[i].offset + (self.levels[i].valid >> INDEX_SHIFT);

      for j in start_idx..end_idx {
        let c1 = self.levels[i - 1].offset + ((j - self.levels[i].offset) << 1);
        let c2 = c1 + 1;

        let e = bits::compact_even_bits(self.flags[c1])
          | (bits::compact_even_bits(self.flags[c2]) << half_shift);
        let o = bits::compact_odd_bits(self.flags[c1])
          | (bits::compact_odd_bits(self.flags[c2]) << half_shift);
        let mask = e & o;

        self.levels[i].avail += bits::ones((!self.flags[j]) & mask);
        self.flags[j] = mask;
      }

      let c1 = self.levels[i - 1].offset + ((end_idx - self.levels[i].offset) << 1);
      let c2 = c1 + 1;

      let e = bits::compact_even_bits(self.flags[c1])
        | (bits::compact_even_bits(self.flags[c2]) << half_shift);
      let o = bits::compact_odd_bits(self.flags[c1])
        | (bits::compact_odd_bits(self.flags[c2]) << half_shift);

      let valid_mask = if end_idx == last_idx {
        1usize.wrapping_shl((self.levels[i].valid & INDEX_MASK) as u32) - 1
      } else {
        usize::MAX
      };

      let mask = e & o & valid_mask;
  
      self.levels[i].avail += bits::ones((!self.flags[end_idx]) & mask);
      self.flags[end_idx] = mask;

      // Moving up to the next level, shift the starting and ending blocks down
      // by one, effectively dividing by two as the block sizes double.
      start >>= 1;
      end >>= 1;
    }
  }

  /// Initialize memory block metadata.
  ///
  /// # Description
  ///
  /// Marks all valid blocks as available.
  fn init_metadata(&mut self) {
    self.flags.fill(usize::MAX);

    for level in &mut self.levels {
      let last = level.valid >> INDEX_SHIFT;
      let rem = level.valid - (last << INDEX_SHIFT);
      self.flags[level.offset + last] = (1 << rem) - 1;
      level.avail = level.valid;
    }
  }

  /// Reserve a block of memory.
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
    let page_size = arch::get_page_size();
    let page_shift = arch::get_page_shift();

    if size == 0 {
      return false;
    }

    if base < self.base {
      return false;
    }

    let last = bits::align_up(base + size, page_size) - 1;

    if last >= self.base + self.size {
      return false;
    }

    let base = bits::align_down(base, page_size);

    let start = (base - self.base) >> page_shift;
    let end = (last - self.base) >> page_shift;

    self.reserve_pages(start, end);

    true
  }

  /// Reserve a block of pages.
  ///
  /// # Parameters
  ///
  /// * `start` - The index of the first page to reserve at level 0.
  /// * `end` - The index of the last page to reserve at level 0.
  ///
  /// # Description
  ///
  /// Reserves the inclusive range [start, end] using relative indices at
  /// level 0.
  ///
  ///   NOTE: No checks are done to ensure this range is valid.
  fn reserve_pages(&mut self, start: usize, end: usize) {
    let mut start = start;
    let mut end = end;

    for level in self.levels.iter_mut() {
      let start_idx = level.offset + (start >> INDEX_SHIFT);
      let end_idx = level.offset + (end >> INDEX_SHIFT);

      // For the first iteration, clear the bits before the start bit.
      let mut mask = usize::MAX << (start & INDEX_MASK);

      // If `start_idx == end_idx`, we'll skip the loop and clear the bits after
      // the end bit. Otherwise, the loop will execute up to, but not including,
      // the end index resetting the mask every time.
      for idx in start_idx..end_idx {
        // Mask off the bits we intend to clear and only reduce the available
        // count by the number of bits that are actually currently one.
        let clear = self.flags[idx] & mask;
        level.avail -= bits::ones(clear);
        self.flags[idx] &= !mask;

        // Reset the mask for the next word.
        mask = usize::MAX;
      }

      // Intersect the mask with the valid bits in the last word.
      mask &= usize::MAX >> (WORD_BITS - (end & INDEX_MASK) - 1);

      // Perform the last iteration of the loop on the final index.
      let clear = self.flags[end_idx] & mask;
      level.avail -= bits::ones(clear);
      self.flags[end_idx] &= !mask;

      // Moving up to the next level, shift the starting and ending blocks down
      // by one, effectively dividing by two as the block sizes double.
      start >>= 1;
      end >>= 1;
    }
  }

  /// Calculates the size of the allocator metadata for the given block size,
  /// and constructs a list of descriptors for the allocator levels.
  ///
  /// # Parameters
  ///
  /// * `size` - The size of the memory block served.
  ///
  /// # Returns
  ///
  /// The a list of descriptors and the allocator metadata size in bytes.
  fn make_levels(size: usize) -> ([PageLevel; PAGE_LEVELS], usize) {
    let page_shift = arch::get_page_shift();

    let mut levels: [PageLevel; PAGE_LEVELS] = Default::default();
    let mut blocks = size >> page_shift;
    let mut offset = 0;

    for i in 0..PAGE_LEVELS {
      levels[i] = PageLevel {
        offset,
        valid: blocks,
        avail: 0,
      };

      offset += (blocks + WORD_BITS - 1) >> INDEX_SHIFT;
      blocks >>= 1;
    }

    (levels, (offset * WORD_BITS) >> 3)
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
  fn find_available_block(&self, level_idx: usize) -> Option<(usize, usize)> {
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
      let bit = bits::least_significant_bit(word);

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
  /// third word at level 2 and its offset is `( 2 << 3 ) + 2 = 18`. The offset
  /// of the starting page at level 0 is `18 << 2 = 72`.
  ///
  /// At level 2, a block is `1 << 2 = 4` pages long. So, the offset of the last
  /// page at level 0 is `((18 + 1) << 2) - 1 = 75`.
  ///
  /// # Returns
  ///
  /// The base address of the allocated block and the number of pages allocated.
  fn allocate_block(&mut self, level_idx: usize, idx: usize, mask: usize) -> (usize, usize) {
    let block = (idx << INDEX_SHIFT) + bits::floor_log2(mask as usize);
    let start = block << level_idx;
    let end = ((block + 1) << level_idx) - 1;

    // Reserve the pages.
    self.reserve_pages(start, end);

    // Return the base address.
    let page_shift = arch::get_page_shift();
    (self.base + (start << page_shift), end - start + 1)
  }
}
