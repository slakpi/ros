//! Buddy Page Allocator
//! https://en.wikipedia.org/wiki/Buddy_memory_allocation

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
  base_addr: usize,
  flags: &'memory mut [u8],
  levels: [PageLevel; PAGE_LEVELS],
}

impl<'memory> PageAllocator<'memory> {
  /// Calculates the size of the allocator structure for the given page size and
  /// memory block size.
  ///
  /// # Parameters
  ///
  /// * `page_size` - The page size in bytes.
  /// * `block_size` - The size of the memory block served.
  ///
  /// # Returns
  ///
  /// The allocator structure size in bytes.
  pub fn calc_size(page_size: usize, block_size: usize) -> usize {
    let (_, size) = PageAllocator::make_levels(page_size, block_size);
    size
  }

  /// Construct a new page allocator.
  ///
  /// # Parameters
  ///
  /// * `page_size` - The page size in bytes.
  /// * `base_addr` - The base address of the contiguous memory block.
  /// * `block_size` - The size of the memory block served.
  /// * `mem` - The memory to use for the allocator struct.
  ///
  /// # Returns
  ///
  /// The allocator structure.
  pub fn new(page_size: usize, base_addr: usize, block_size: usize, mem: *mut u8) -> Self {
    let (levels, size) = PageAllocator::make_levels(page_size, block_size);
    let mut allocator = PageAllocator {
      page_size,
      base_addr,
      flags: unsafe { slice::from_raw_parts_mut(mem, size) },
      levels,
    };

    // Initialize the availability flags. Any blocks not covered by the level
    // above will be marked as available.
    let mut bits = block_size / page_size;

    allocator.flags.fill(0);

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
      let mut idx = allocator.levels[i].offset + (bit >> 3);
      let mut mask = (bit & 0x7) as u8;

      // If this is the last possible page level or the next level has no valid
      // blocks, set all blocks as available.
      if (i == PAGE_LEVELS - 1) || (allocator.levels[i + 1].valid == 0) {
        bit = 0;
        idx = allocator.levels[i].offset;
        mask = 1;
      }

      for _ in bit..allocator.levels[i].valid {
        allocator.flags[idx] |= mask;

        mask <<= 1;

        if mask == 0 {
          idx += 1;
          mask = 1;
        }
      }

      allocator.levels[i].avail = allocator.levels[i].valid - bit;
    }

    allocator
  }

  /// Calculates the size of the allocator structure for the given page size and
  /// memory block size.
  ///
  /// # Parameters
  ///
  /// * `page_size` - The page size in bytes.
  /// * `block_size` - The size of the memory block served.
  ///
  /// # Returns
  ///
  /// The allocator structure size in bytes.
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
    //   |   |   |   |   |   |   |   |   |   |   |   |   |   |   |   | / |  0
    //   +---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+
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
    //   |                               | ///////////////////////////// |  3
    //   +---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+
    //                                                                   2
    //
    //   +---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+
    //   |               |               |               | ///////////// |  2
    //   +---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+
    //                                                                   4
    //
    //   +---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+
    //   |       |       |       |       |       |       |       | ///// |  1
    //   +---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+
    //                                                                   8
    //
    //   +---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+
    //   |   |   |   |   |   |   |   |   |   |   |   |   |   |   |   | / |  0
    //   +---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+---+
    //                                                                  16
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
}
