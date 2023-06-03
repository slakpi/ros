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

    let (levels, alloc_size) = PageAllocator::make_levels(page_size, size);
    let size = bits::align_down(size, page_size);
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
  /// `pages` - The number of pages to allocate.
  ///
  /// # Returns
  ///
  /// Ok with the starting physical address of the block if a contiguous block
  /// is found. Err if a large enough contiguous block cannot be found or the
  /// requested page count exceeds 2^(PAGE_LEVELS - 1).
  pub fn allocate(&mut self, pages: usize) -> Result<usize, ()> {
    // Calculate the level with the ideal block size.
    let min_level = bits::ceil_log2(pages);

    // Find the smallest available block.
    if let Ok((level, idx, mask)) = self.find_available_block(min_level) {
      // Allocate the block by splitting as necessary.
      return Ok(self.allocate_block(min_level, level, idx, mask));
    }

    // Sorry, try another allocator or do some swapping.
    Err(())
  }

  pub fn free(&mut self, base: usize, size: usize) {}

  /// Initialize memory block metadata.
  ///
  /// # Description
  ///
  /// All valid blocks at `PAGE_LEVELS - 1` are initialized to available.
  ///
  /// At lower levels, any blocks not covered by the level above are marked as
  /// available. For example:
  ///
  ///   +---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+
  ///   | / | / | / | / | / | / | / | / | / | / | / | / | / | / | * | x |  0
  ///   +---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+
  ///   0                               8                       14  15 16
  ///
  ///   +---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+
  ///   | ///// | ///// | ///// | ///// | ///// | ///// | ***** | xxxxx |  1
  ///   +---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+
  ///   0                               4               6       7       8
  ///
  ///   +---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+
  ///   | ///////////// | ///////////// | ************* | xxxxxxxxxxxxx |  2
  ///   +---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+
  ///   0                               2               3               4
  ///
  ///   +---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+
  ///   | ***************************** | xxxxxxxxxxxxxxxxxxxxxxxxxxxxx | 3
  ///   +---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+
  ///   0                               1                               2
  ///
  /// The allocator covers 15 4 KiB pages. This means there are only 7 8 KiB
  /// blocks possible covering 14 4 KiB pages. The 15th page is marked
  /// available. Likewise, there are only 3 16 KiB blocks available covering 6
  /// 8 KiB blocks, so the 7th 8 KiB block is marked available. And so on.
  fn init_flags(&mut self) {
    let mut bits = self.size / self.page_size;

    self.flags.fill(0);

    for i in 0..PAGE_LEVELS {
      bits >>= 1;

      // If `bits` is not a power of two, shifting it left again will give the
      // bit index of the uncovered bits. For example, 15 >> 1 = 7, then
      // 7 << 1 = 14. Starting from that bit index, mark pages as available up
      // to the valid number of bits in the level.
      //
      // Shifting the bit index down by 3 gives the start byte index, and a
      // modulo 7 gives the bit index within that byte. E.g. 14 >> 3 = 1, so we
      // start with byte 1. 14 & 7 = 6, so start with bit 6.
      let mut bit = bits << 1;
      let mut idx = self.levels[i].offset + (bit >> 3);
      let mut mask = (1 << (bit & 0x7)) as u8;

      // If this is the last possible page level or the next level has no valid
      // blocks, set all blocks as available.
      if (i == PAGE_LEVELS - 1) || (self.levels[i + 1].valid == 0) {
        bit = 0;
        idx = self.levels[i].offset;
        mask = 0x1;
      }

      for _ in bit..self.levels[i].valid {
        self.flags[idx] |= mask;

        mask <<= 1;

        if mask == 0 {
          idx += 1;
          mask = 0x1;
        }
      }

      self.levels[i].avail = self.levels[i].valid - bit;
    }
  }

  /// Reserve a block of memory.
  ///
  /// # Parameters
  ///
  /// * `base` - The address of the block to reserve.
  /// * `size` - The size of the block to reserve.
  ///
  ///
  /// # Description
  ///
  /// Reservation starts working with single pages and works up to the largest
  /// block size.
  ///
  /// Let's say we want to reserve 16 KiB (four 4 KiB pages) starting at
  /// address 12 shown with / in the diagram below:
  ///
  ///   +---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+
  ///   |   |   |   | / | / | / | / |   |   |   |   |   |   |   |   |   |  0
  ///   +---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+
  ///   0       2   3   4   5   6   7   8                              16
  ///
  /// Pages 2 and 3 are buddies. 3 is the odd buddy, so we are implicitly
  /// splitting an 8 KiB block at the next level. We can mark page 2 as
  /// available, the proceed to mark pages 3, 4, 5, and 6 as unavailable.
  ///
  /// Pages 6 and 7 are buddies. 6 is the even buddy, so we are implicitly
  /// splitting another 8 KiB block at the next level and can mark page 7 as
  /// available.
  ///
  /// In all cases, the available count only changes if the state of a page
  /// changes. If an unavailable page becomes available, the available count
  /// increments and vice versa. This allows for overlapping reservations.
  ///
  /// The final state of Level 0 is:
  ///
  ///   +---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+
  ///   |   |   | * | / | / | / | / | * |   |   |   |   |   |   |   |   |  0
  ///   +---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+
  ///   0       2   3   4   5   6   7   8                              16
  ///
  /// * is an available page, / is an unavailable page, and the remaining pages
  /// are unchanged.
  ///
  /// Next, shift the start and end down by 1 bit, then repeat:
  ///
  ///   3 >> 1 = 1 and 6 >> 1 = 3
  ///   +---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+
  ///   | ***** | ///// | ///// | ///// |       |       |       |       |  1
  ///   +---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+
  ///   0       1       2       3       4                               8
  ///
  ///   1 >> 1 = 0 and 3 >> 1 = 1
  ///   +---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+
  ///   | ///////////// | ///////////// |               |               |  2
  ///   +---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+
  ///   0                               2                               4
  ///
  ///   0 >> 1 = 0 and 1 >> 1 = 0
  ///   +---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+
  ///   | ///////////////////////////// | ***************************** |  3
  ///   +---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+
  ///   0                               1                               2
  ///
  /// # Returns
  ///
  /// True if the block resides completely within the area served by this
  /// allocator, false otherwise.
  fn reserve(&mut self, base: usize, size: usize) -> bool {
    let last = base + size - 1;

    if size == 0 {
      return false;
    }

    if (base < self.base) || (last >= self.base + self.size) {
      return false;
    }

    // Find the page indices for the start and end of the reservation. Round the
    // start down and round the end up.
    let mut start = base >> self.page_shift;
    let mut end = last >> self.page_shift;

    // Start from Level 0 and work up.
    for l in 0..PAGE_LEVELS {
      let mut idx = self.levels[l].offset + (start >> 3);
      let mut mask = (1 << (start & 0x7)) as u8;

      // If the start index is odd, mark its even buddy as available. Only
      // increment the available page count if the even buddy was not already
      // marked available.
      if (start & 0x1) != 0 {
        self.levels[l].avail += ((self.flags[idx] & (mask >> 1)) == 0) as usize;
        self.flags[idx] |= mask >> 1;
      }

      // Mark the occupied block as unavailable. Only decrement the available
      // count if the block was marked available.
      for _ in start..=end {
        self.levels[l].avail -= ((self.flags[idx] & mask) != 0) as usize;
        self.flags[idx] &= !mask;
        mask <<= 1;

        if mask == 0 {
          idx += 1;
          mask = 0x1;
        }
      }

      // If the end index is even, mark its odd buddy as available. Only
      // increment the available page count if the odd buddy was not already
      // marked available. Note: `idx` and `mask` have already been updated to
      // the correct position by the loop.
      if (end & 0x1) == 0 {
        self.levels[l].avail += ((self.flags[idx] & mask) == 0) as usize;
        self.flags[idx] |= mask << 1;
      }

      start >>= 1;
      end >>= 1;
    }

    true
  }

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
  ///
  /// # Assumes
  ///
  /// Assumes that the page size has already been validated for the
  /// architecture.
  fn make_levels(page_size: usize, block_size: usize) -> ([PageLevel; PAGE_LEVELS], usize) {
    let mut levels: [PageLevel; PAGE_LEVELS] = Default::default();

    // Calculate the number of pages in the block. We're rounding down, so any
    // bytes that do not make up a full page will be ignored.
    let mut bits = block_size / page_size;

    // Level 0, the actual number of pages in the block and the number of bytes
    // to represent each page as a single bit.
    //
    // Take the simple case of 15 pages. Each page is a bit, so:
    //
    //   ceil( 15 / 8 ) = 2
    //
    // Level 0 will require 2 bytes where 15 of the bits will be valid:
    //
    //   +---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+
    //   |   |   |   |   |   |   |   |   |   |   |   |   |   |   |   | x |  0
    //   +---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+
    //   0                               8                              16
    let mut size = (bits + 7) >> 3;
    levels[0] = PageLevel {
      offset: 0,
      valid: bits,
      avail: 0,
    };

    // Now add the size of subsequent levels. For each level, shift the number
    // of bits down by one and calculate ceil( bits / 8 ).
    //
    //   +---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+
    //   |       |       |       |       |       |       |       | xxxxx |  1
    //   +---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+
    //   0                               4                               8
    //
    //   +---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+
    //   |               |               |               | xxxxxxxxxxxxx |  2
    //   +---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+
    //   0                               2                               4
    //
    //   +---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+
    //   |                               | xxxxxxxxxxxxxxxxxxxxxxxxxxxxx |  3
    //   +---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+
    //   0                               1                               2
    //
    // In total, level 0 requires 2 bytes; levels 1, 2, and 3 require 1 byte
    // each. Levels 0 - 3 have 15, 7, 3, and 1 valid bit(s) respectively. Levels
    // 4 up to PAGE_LEVELS will just have 0 valid bits and contribute 0 bytes to
    // the total size.
    //
    // This translates to:
    //
    //   * 1 possible block of 8 pages.
    //   * 3 possible blocks of 4 pages.
    //   * 7 possible blocks of 2 pages.
    //   * 15 possible blocks of 1 page.
    for i in 1..PAGE_LEVELS {
      bits = bits >> 1;
      levels[i] = PageLevel {
        offset: size,
        valid: bits,
        avail: 0,
      };

      size += (bits + 7) >> 3;
    }

    (levels, size)
  }

  /// Finds the first available block.
  ///
  /// # Parameters
  ///
  /// `min_level` - The minimum level. Start the search from here.
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
  fn find_available_block(&self, min_level: usize) -> Result<(usize, usize, u8), ()> {
    for l in min_level..PAGE_LEVELS {
      if self.levels[l].avail == 0 {
        continue;
      }

      let mut idx = self.levels[l].offset;
      let mut block = 0;

      while block < self.levels[l].valid {
        if (self.flags[idx] & 0xff) == 0 {
          idx += 1;
          block += 8;
          continue;
        }

        let mask = bits::least_significant_bit(self.flags[idx] as usize);
        return Ok((l, idx, mask as u8));
      }

      // There is something wrong with the availability accounting or there are
      // no valid blocks. Either case is a panic.
      debug_assert!(false);
      break;
    }

    Err(())
  }

  /// Allocate a block.
  ///
  /// # Parameters
  ///
  /// `min_level` - The minimum block size.
  /// `level` - The level with an available block.
  /// `idx` - The byte index of the available block.
  /// `mask` - The byte mask of the available block.
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
    let mut idx = idx;
    let mut alloc_mask = mask;
    let mut avail_mask = 0u8;

    for l in (min_level..=level).rev() {
      // Update the availability flags at this level.
      self.flags[idx] &= !alloc_mask;
      self.flags[idx] |= avail_mask;

      // If `avail_mask` is zero, we are only allocating a block. Otherwise, we
      // splitting. This adds two blocks to the level and allocates one.
      if avail_mask == 0 {
        self.levels[l].avail -= 1;
      } else {
        self.levels[l].avail += 1;
      }

      // If we're at the minimum level, there's nothing left to do.
      if l == min_level {
        break;
      }

      // Calculate the block number in the next level down, then set the masks.
      // By definition, the buddy blocks will be in the same byte. It does not
      // matter which block we allocate, so just allocate the even block.
      let rel_idx = idx - self.levels[l].offset;
      let block = ((rel_idx << 3) + bits::floor_log2(alloc_mask as usize)) << 1;
      idx = (block >> 3) + self.levels[l - 1].offset;
      alloc_mask = (1 << (block & 0x7)) as u8;
      avail_mask = alloc_mask << 1;
    }

    // Calculate the page number from the block number.
    let rel_idx = idx - self.levels[min_level].offset;
    let block = ((rel_idx << 3) + bits::floor_log2(alloc_mask as usize)) << 1;
    let page = block << level;

    // Calculate and return the base address of the block.
    self.base + (page << self.page_shift)
  }
}
