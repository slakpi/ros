use super::{PageAllocator, PageLevel};
use core::iter;

// Test with 4 KiB pages.
const TEST_PAGE_SIZE: usize = 4096;

// Test with 2047 pages. The non-power of 2 tests proper setup and accounting.
// At each level below 2^10, there should be one available block. For example,
// There can only be 1 block of 1024 pages and 3 blocks of 512. The block of
// 1024 covers two of the blocks of 512, leaving the last block of 512
// available.
const TEST_MEM_SIZE: usize = TEST_PAGE_SIZE * 2047;

// Block Size (Pages)       Bytes Required      Valid Bits      Available
// ----------------------------------------------------------------------
// 1024                       1                    1            1
//  512                       1                    3            1
//  256                       1                    7            1
//  128                       2                   15            1
//   64                       4                   31            1
//   32                       8                   63            1
//   16                      16                  127            1
//    8                      32                  255            1
//    4                      64                  511            1
//    2                     128                 1023            1
//    1                     256                 2047            1
// ----------------------------------------------------------------------
//                          513 bytes total for metadata
const EXPECTED_METADATA_SIZE: usize = 513;

const EXPECTED_PAGE_LEVELS: usize = 11;

const EXPECTED_LEVELS: [PageLevel; EXPECTED_PAGE_LEVELS] = [
  PageLevel {
    offset: 0,
    valid: 2047,
    avail: 1,
  },
  PageLevel {
    offset: 256,
    valid: 1023,
    avail: 1,
  },
  PageLevel {
    offset: 384,
    valid: 511,
    avail: 1,
  },
  PageLevel {
    offset: 448,
    valid: 255,
    avail: 1,
  },
  PageLevel {
    offset: 480,
    valid: 127,
    avail: 1,
  },
  PageLevel {
    offset: 496,
    valid: 63,
    avail: 1,
  },
  PageLevel {
    offset: 504,
    valid: 31,
    avail: 1,
  },
  PageLevel {
    offset: 508,
    valid: 15,
    avail: 1,
  },
  PageLevel {
    offset: 510,
    valid: 7,
    avail: 1,
  },
  PageLevel {
    offset: 511,
    valid: 3,
    avail: 1,
  },
  PageLevel {
    offset: 512,
    valid: 1,
    avail: 1,
  },
];

pub fn run_tests() {
  test_size_calculation();
  test_level_construction();
}

fn test_size_calculation() {
  let size = PageAllocator::calc_size(TEST_PAGE_SIZE, TEST_MEM_SIZE);
  assert!(
    size == EXPECTED_METADATA_SIZE,
    "Calculated size was {} instead of {}.",
    size,
    EXPECTED_METADATA_SIZE
  );
}

fn test_level_construction() {
  let (levels, _) = PageAllocator::make_levels(TEST_PAGE_SIZE, TEST_MEM_SIZE);
  assert!(
    levels.len() == EXPECTED_LEVELS.len(),
    "Level count mismatch: expected {}, received: {}",
    EXPECTED_LEVELS.len(),
    levels.len(),
  );

  for (a, b) in iter::zip(EXPECTED_LEVELS, levels) {
    assert!(
      a.offset == b.offset,
      "Offset mismatch: expected {}, received: {}",
      a.offset,
      b.offset,
    );

    assert!(
      a.valid == b.valid,
      "Valid bit count mismatch: expected {}, received: {}",
      a.valid,
      b.valid,
    );

    // `make_levels` does not determine the number of available blocks at each
    // level. That is done when initializing the allocator's metadata.
    assert!(
      b.avail == 0,
      "Available bit count mismatch: expected {}, received: {}",
      0,
      b.avail,
    );
  }
}

fn test_allocator_construction() {
  let mut buffer: [u8; EXPECTED_METADATA_SIZE] = [0xff; EXPECTED_METADATA_SIZE];
  let mem = buffer.as_mut_ptr();
  
}