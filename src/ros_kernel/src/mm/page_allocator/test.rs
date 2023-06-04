use super::{PageAllocator, PageLevel};
use crate::arch::bits;
use crate::peripherals::memory;
use crate::test;
use crate::{check_eq, debug_print, execute_test};
use core::{iter, slice};

struct AvailableBlocks<'a> {
  levels: [&'a [usize]; EXPECTED_PAGE_LEVELS],
}

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
  execute_test!(test_size_calculation);
  execute_test!(test_level_construction);
  execute_test!(test_flag_init);
  execute_test!(test_reservation_errors);
  execute_test!(test_reservations);
}

fn test_size_calculation(context: &mut test::TestContext) {
  let size = PageAllocator::calc_size(TEST_PAGE_SIZE, TEST_MEM_SIZE);
  check_eq!(context, size, EXPECTED_METADATA_SIZE);
}

fn test_level_construction(context: &mut test::TestContext) {
  let (levels, _) = PageAllocator::make_levels(TEST_PAGE_SIZE, TEST_MEM_SIZE);
  check_eq!(context, levels.len(), EXPECTED_LEVELS.len());

  for (a, b) in iter::zip(levels, EXPECTED_LEVELS) {
    check_eq!(context, a.offset, b.offset);
    check_eq!(context, a.valid, b.valid);

    // `make_levels` does not determine the number of available blocks at each
    // level. That is done when initializing the allocator's metadata.
    check_eq!(context, a.avail, 0);
  }
}

fn test_flag_init(context: &mut test::TestContext) {
  // Initialize all bytes in the metadata buffer to 0xff to ensure flag
  // initialization sets them appropriately. No bytes should be 0xff after
  // initialization.
  let mut buffer: [u8; EXPECTED_METADATA_SIZE] = [0xff; EXPECTED_METADATA_SIZE];

  // Get a page allocator.
  let mut allocator = make_allocator(0x2000, buffer.as_mut_ptr());

  // Test flag initialization.
  allocator.init_flags();

  // Verify the availability counts match.
  for (a, b) in iter::zip(&allocator.levels, EXPECTED_LEVELS) {
    check_eq!(context, a.avail, b.avail);
  }

  // At level 10, the first and only block of 1024 is available. At level 9, the
  // third and last block of 512 is available. At all lower levels, the seventh
  // and last block is available. For example, at level 0, 8 * 255 = 2040 and
  // page 2047 should be available, so bit 7 should be set. At level 7,
  // 8 * 1 = 8 and block 15 should be available, so bit 7 should be set again.
  // Etc.
  verify_available_blocks(
    context,
    "Flag init",
    &allocator,
    AvailableBlocks {
      levels: [
        &[2047],
        &[1023],
        &[511],
        &[255],
        &[127],
        &[63],
        &[31],
        &[15],
        &[7],
        &[3],
        &[1],
      ],
    },
  );
}

fn test_reservation_errors(context: &mut test::TestContext) {
  const TEST_ALLOC_BASE: usize = 0x8000;

  let mut a_buff: [u8; EXPECTED_METADATA_SIZE] = [0xff; EXPECTED_METADATA_SIZE];
  let mut b_buff: [u8; EXPECTED_METADATA_SIZE] = [0xfe; EXPECTED_METADATA_SIZE];
  let mut a = make_allocator(TEST_ALLOC_BASE, a_buff.as_mut_ptr());
  let mut b = make_allocator(TEST_ALLOC_BASE, b_buff.as_mut_ptr());

  a.init_flags();
  b.init_flags();

  // Reserving more memory than served by the allocator should fail without
  // changing the allocator's state.
  check_eq!(
    context,
    a.reserve(TEST_ALLOC_BASE, TEST_MEM_SIZE + 1),
    false
  );
  check_eq!(context, compare_allocators(&a, &b), true);

  // Reserving memory that overlaps the beginning of the area served by the
  // allocator should fail without changing the allocator's state.
  check_eq!(
    context,
    a.reserve(TEST_ALLOC_BASE - TEST_PAGE_SIZE, TEST_PAGE_SIZE * 2),
    false
  );
  check_eq!(context, compare_allocators(&a, &b), true);

  // Reserving memory that overlaps the end of the area served by the allocator
  // should fail without changing the allocator's state.
  check_eq!(
    context,
    a.reserve(
      TEST_ALLOC_BASE + TEST_MEM_SIZE - TEST_PAGE_SIZE,
      TEST_PAGE_SIZE * 2
    ),
    false
  );
  check_eq!(context, compare_allocators(&a, &b), true);
}

fn test_reservations(context: &mut test::TestContext) {
  const TEST_ALLOC_BASE: usize = 0x80000;

  let mut buffer: [u8; EXPECTED_METADATA_SIZE] = [0xff; EXPECTED_METADATA_SIZE];
  let mut allocator = make_allocator(TEST_ALLOC_BASE, buffer.as_mut_ptr());

  allocator.init_flags();

  // Reserve a 0xe000 byte block starting at 0x2001. Page-alignment forces
  // reservation of 0x2000 - 0x10fff (Pages 3 - 17).
  allocator.reserve(TEST_ALLOC_BASE + 0x2001, 0xe000);
  verify_available_blocks(
    context,
    "Reservation step 1",
    &allocator,
    AvailableBlocks {
      levels: [
        &[18, 2047],
        &[1, 10, 1023], // Split block 9
        &[6, 511],      // Split blocks 1 and 5
        &[4, 255],      // Split blocks 1 and 3
        &[127],         // Split blocks 1 and 2
        &[2, 63],       // Split block 1
        &[2, 31],       // Split block 1
        &[2, 15],       // Split block 1
        &[2, 7],        // Split block 1
        &[2, 3],        // Split block 1
        &[],            // Split block 1
      ],
    },
  );

  // Without resetting the flags, reserve a 0x8000 byte block starting at
  // 0x12001. Page-alignment forces reservation of 0x12000 - 0x1afff (Pages
  // 19 - 27). There should be no changes in levels 10 down to 4.
  // Level 3: Split block 4, block 255 is available.
  // Level 2: Consume block 6 and split block 7, blocks 8 and 511 are available.
  // Level 1: Consume block 10 and split block 14, blocks 1 and 1023 are available.
  // Level 0: Pages 18, 28, and 2047 are available.
  allocator.reserve(TEST_ALLOC_BASE + 0x12001, 0x8000);
  verify_available_blocks(
    context,
    "Reservation step 2",
    &allocator,
    AvailableBlocks {
      levels: [
        &[18, 28, 2047],
        &[1, 1023],      // Consume block 10 and split block 14
        &[8, 511],       // Consume block 6 and split block 7
        &[255],          // Split block 4
        &[127],          // No change
        &[2, 63],        // No change
        &[2, 31],        // No change
        &[2, 15],        // No change
        &[2, 7],         // No change
        &[2, 3],         // No change
        &[],             // No change
      ],
    },
  );

  // Without resetting the flags, reserve a 0x8000 byte block starting at
  // 0xd001. Page-alignment forces reservation of 0xd000 - 0x15fff (Pages
  // 14 - 22). This overlaps the previous two ranges. There should be no changes
  // in levels 10 down to 1.
  // Level 0: Consume page 18, pages 28 and 2047 are available.
  // allocator.reserve(0xd001, 0x8000);
}

/// Construct an allocator without initializing the metadata flags.
///
/// # Parameters
///
/// * `base` - The base address to use.
/// * `mem` - A memort block of at least TEST_MEM_SIZE bytes.
///
/// # Returns
///
/// A partially initialized allocator.
fn make_allocator<'memory>(base: usize, mem: *mut u8) -> PageAllocator<'memory> {
  let (levels, size) = PageAllocator::make_levels(TEST_PAGE_SIZE, TEST_MEM_SIZE);

  // Manually create a PageAllocator to prevent it from performing any flag
  // initializations or memory reservations.
  PageAllocator {
    page_size: TEST_PAGE_SIZE,
    page_shift: bits::floor_log2(TEST_PAGE_SIZE),
    base,
    size: TEST_MEM_SIZE,
    flags: unsafe { slice::from_raw_parts_mut(mem, size) },
    levels,
  }
}

/// Verifies the allocator metadata given a list of expected available blocks.
///
/// # Parameters
///
/// * `context` - The testing context.
/// * `tag` - A tag to include in check statements.
/// * `allocator` - The allocator to verify.
/// * `exp_avail` - Expected availability state.
///
/// # Description
///
/// Validates that each level in the allocator has the same number of available
/// blocks as specified by the corresponding expected list.
///
/// Validates that each level's metadata has only the bits specified by the
/// block numbers in the corresponding expected list set.
///
///   Note: Block numbers are 1-based. The function will assert if zero is
///         encountered.
fn verify_available_blocks(
  context: &mut test::TestContext,
  tag: &str,
  allocator: &PageAllocator,
  exp_avail: AvailableBlocks,
) {
  for (level, exp) in iter::zip(&allocator.levels, exp_avail.levels) {
    check_eq!(context, level.avail, exp.len(), tag);

    let last = level.valid >> 3;
    let end = level.offset + last;
    let mut exp_idx = 0;

    for idx in level.offset..=end {
      let mut mask = 0;

      while exp_idx < exp.len() {
        assert!(exp[exp_idx] > 0);
        let block = exp[exp_idx] - 1;
        let tmp_idx = level.offset + (block >> 3);

        if tmp_idx > idx {
          break;
        } else if tmp_idx == idx {
          mask |= 1 << (block & 0x7);
        }

        exp_idx += 1;
      }

      debug_print!("  {} level {}\n", tag, level.offset);
      check_eq!(context, allocator.flags[idx], mask, tag);
    }
  }
}

/// Deep compare of two allocators.
///
/// # Parameters
///
/// * `act` - The actual allocator state.
/// * `exp` - The expected allocator state.
///
/// # Returns
///
/// True if the allocator metadata matches exactly, false otherwise.
fn compare_allocators(act: &PageAllocator, exp: &PageAllocator) -> bool {
  for (a, e) in iter::zip(&act.levels, &exp.levels) {
    if a.offset != e.offset || a.valid != e.valid || a.avail != e.avail {
      return false;
    }

    let last = a.offset + (a.valid >> 3);

    if act.flags[a.offset..=last] != exp.flags[e.offset..=last] {
      return false;
    }
  }

  true
}
