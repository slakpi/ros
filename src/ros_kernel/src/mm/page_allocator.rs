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

const INDEX_SHIFT: usize = bits::floor_log2(u8::BITS as usize);

/// Metadata for each level in the buddy allocator.
#[derive(Default)]
struct PageLevel {
  offset: usize,
  valid: usize,
  avail: usize,
}

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
  /// `make_levels` should have been called to ensure that `mem` has sufficient
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
    // for r in excl.get_ranges() {
    //   _ = allocator.reserve(r.base, r.size);
    // }

    // Reserve the allocator's own metadata memory.
    // let mem_addr = (mem as usize) - arch::get_kernel_virtual_base();
    // _ = allocator.reserve(mem_addr, alloc_size);

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
  /// is found. Err if a large enough contiguous block cannot be found or the
  /// requested page count exceeds 2^(PAGE_LEVELS - 1).
  pub fn allocate(&mut self, pages: usize) -> Option<usize> {
    // // Calculate the level with the ideal block size.
    // let min_level = bits::ceil_log2(pages);

    // // Find the smallest available block.
    // if let Ok((level, idx, mask)) = self.find_available_block(min_level) {
    //   // Allocate the block by splitting as necessary.
    //   return Ok(self.allocate_block(min_level, level, idx, mask));
    // }

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

    let mut start = (base - self.base) >> self.page_shift;
    let mut end = (last - self.base) >> self.page_shift;

    for level in self.levels.iter_mut() {
      let mut idx = level.offset + (start >> 3);
      let mut mask = 1 << (start & 0x7);

      for _ in start..=end {
        level.avail -= ((self.flags[idx] & mask) != 0) as usize;
        self.flags[idx] &= !mask;
        mask <<= 1;

        if mask == 0 {
          idx += 1;
          mask = 1;
        }
      }

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

      offset += (blocks + 7) >> 3;
      blocks >>= 1;
    }

    (levels, offset)
  }

  /// Finds the first available block.
  ///
  /// # Parameters
  ///
  /// * `min_level` - The minimum level. Start the search from here.
  ///
  /// # Description
  ///
  /// Searches upward, starting from `min_level`, for the smallest available
  /// block that is at least `2^min_level` pages.
  ///
  /// # Returns
  ///
  /// Ok with a tuple containing the level at which the block was found, the
  /// index of the byte with the available block, and a byte mask identifying
  /// the available block's bit. Err if there is no block available or
  /// `min_level` is too large.
  fn find_available_block(&self, min_level: usize) -> Option<(usize, usize, u8)> {
    // for l in min_level..PAGE_LEVELS {
    //   if self.levels[l].avail == 0 {
    //     continue;
    //   }

    //   let mut idx = self.levels[l].offset;
    //   let mut block = 0;

    //   while block < self.levels[l].valid {
    //     if (self.flags[idx] & 0xff) == 0 {
    //       idx += 1;
    //       block += 8;
    //       continue;
    //     }

    //     let mask = bits::least_significant_bit(self.flags[idx] as usize);
    //     return Ok((l, idx, mask as u8));
    //   }

    //   // There is something wrong with the availability accounting or there are
    //   // no valid blocks. Either case is a panic.
    //   debug_assert!(false);
    //   break;
    // }

    // Err(())

    None
  }

  /// Allocate a block.
  ///
  /// # Parameters
  ///
  /// * `min_level` - The minimum block size.
  /// * `level` - The level with an available block.
  /// * `idx` - The byte index of the available block.
  /// * `mask` - The byte mask of the available block.
  ///
  /// # Description
  ///
  /// If `level` = `min_level`, the `allocate_block` simply clears the block's
  /// bit and updates the availability count. Otherwise, splitting needs to be
  /// done.
  ///
  /// # Returns
  ///
  /// The base address of the allocated block.
  fn allocate_block(&mut self, min_level: usize, level: usize, idx: usize, mask: u8) -> usize {
    // let mut idx = idx;
    // let mut alloc_mask = mask;
    // let mut avail_mask = 0u8;

    // for l in (min_level..=level).rev() {
    //   // Update the availability flags at this level.
    //   self.flags[idx] &= !alloc_mask;
    //   self.flags[idx] |= avail_mask;

    //   // If `avail_mask` is zero, we are only allocating a block. Otherwise, we
    //   // splitting. This adds two blocks to the level and allocates one.
    //   if avail_mask == 0 {
    //     self.levels[l].avail -= 1;
    //   } else {
    //     self.levels[l].avail += 1;
    //   }

    //   // If we're at the minimum level, there's nothing left to do.
    //   if l == min_level {
    //     break;
    //   }

    //   // Calculate the block number in the next level down, then set the masks.
    //   // By definition, the buddy blocks will be in the same byte. It does not
    //   // matter which block we allocate, so just allocate the even block.
    //   let rel_idx = idx - self.levels[l].offset;
    //   let block = ((rel_idx << 3) + bits::floor_log2(alloc_mask as usize)) << 1;
    //   idx = (block >> 3) + self.levels[l - 1].offset;
    //   alloc_mask = 1 << (block & 0x7);
    //   avail_mask = alloc_mask << 1;
    // }

    // // Calculate the page number from the block number.
    // let rel_idx = idx - self.levels[min_level].offset;
    // let block = ((rel_idx << 3) + bits::floor_log2(alloc_mask as usize)) << 1;
    // let page = block << level;

    // // Calculate and return the base address of the block.
    // self.base + (page << self.page_shift)

    0
  }
}
