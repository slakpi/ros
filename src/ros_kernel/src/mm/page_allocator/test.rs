use super::{PageAllocator, PageLevel};
use crate::arch::bits;
use crate::peripherals::memory;
use crate::test;
use crate::{check_eq, debug_print, execute_test};
use core::{iter, slice};

/// Represents an allocator state usings lists of closed, 1-based ranges of
/// allocated blocks at each level.
struct AllocatorState<'a> {
  levels: [&'a [(usize, usize)]; EXPECTED_PAGE_LEVELS],
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
    avail: 2047,
  },
  PageLevel {
    offset: 256,
    valid: 1023,
    avail: 1023,
  },
  PageLevel {
    offset: 384,
    valid: 511,
    avail: 511,
  },
  PageLevel {
    offset: 448,
    valid: 255,
    avail: 255,
  },
  PageLevel {
    offset: 480,
    valid: 127,
    avail: 127,
  },
  PageLevel {
    offset: 496,
    valid: 63,
    avail: 63,
  },
  PageLevel {
    offset: 504,
    valid: 31,
    avail: 31,
  },
  PageLevel {
    offset: 508,
    valid: 15,
    avail: 15,
  },
  PageLevel {
    offset: 510,
    valid: 7,
    avail: 7,
  },
  PageLevel {
    offset: 511,
    valid: 3,
    avail: 3,
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
  const LAST_BYTES: [u8; EXPECTED_PAGE_LEVELS] = [
    0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7f, 0x7, 0x1,
  ];

  // Initialize all bytes in the metadata buffer to 0 to ensure flag
  // initialization sets them appropriately. No bytes should be zero after
  // initialization.
  let mut buffer: [u8; EXPECTED_METADATA_SIZE] = [0; EXPECTED_METADATA_SIZE];

  // Get a page allocator.
  let mut allocator = make_allocator(0x2000, buffer.as_mut_ptr());

  // Test flag initialization.
  allocator.init_flags();

  // Verify the availability counts match.
  for (a, b) in iter::zip(&allocator.levels, EXPECTED_LEVELS) {
    check_eq!(context, a.avail, b.avail);
    check_eq!(context, a.avail, a.valid);
  }

  // Verify the availability flags.
  for (level, last_byte) in iter::zip(&allocator.levels, &LAST_BYTES) {
    let last = level.valid >> 3;

    for i in 0..last {
      check_eq!(context, allocator.flags[level.offset + i], 0xff);
    }

    check_eq!(context, allocator.flags[level.offset + last], *last_byte);
  }
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

  let mut buffer: [u8; EXPECTED_METADATA_SIZE] = [0; EXPECTED_METADATA_SIZE];
  let mut allocator = make_allocator(TEST_ALLOC_BASE, buffer.as_mut_ptr());

  allocator.init_flags();

  // Reserve a 0xe000 byte block starting at 0x2001. Page-alignment forces
  // reservation of 0x2000 - 0x10fff (Pages 3 - 17).
  allocator.reserve(TEST_ALLOC_BASE + 0x2001, 0xe000);
  verify_allocated_blocks(
    context,
    "Reservation step 1",
    &allocator,
    AllocatorState {
      levels: [
        &[(3, 17)],
        &[(2, 9)],
        &[(1, 5)],
        &[(1, 3)],
        &[(1, 2)],
        &[(1, 1)],
        &[(1, 1)],
        &[(1, 1)],
        &[(1, 1)],
        &[(1, 1)],
        &[(1, 1)],
      ],
    },
  );

  // Without resetting the flags, reserve a 0x8000 byte block starting at
  // 0x12001. Page-alignment forces reservation of 0x12000 - 0x1afff (Pages
  // 19 - 27).
  allocator.reserve(TEST_ALLOC_BASE + 0x12001, 0x8000);
  verify_allocated_blocks(
    context,
    "Reservation step 2",
    &allocator,
    AllocatorState {
      levels: [
        &[(3, 17), (19, 27)],
        &[(2, 14)],
        &[(1, 7)],
        &[(1, 4)],
        &[(1, 2)],
        &[(1, 1)],
        &[(1, 1)],
        &[(1, 1)],
        &[(1, 1)],
        &[(1, 1)],
        &[(1, 1)],
      ],
    },
  );

  // Without resetting the flags, reserve a 0x8000 byte block starting at
  // 0xd001. Page-alignment forces reservation of 0xd000 - 0x15fff (Pages
  // 14 - 22). This overlaps the previous two ranges.
  allocator.reserve(TEST_ALLOC_BASE + 0xd001, 0x8000);
  verify_allocated_blocks(
    context,
    "Reservation step 3",
    &allocator,
    AllocatorState {
      levels: [
        &[(3, 27)],
        &[(2, 14)],
        &[(1, 7)],
        &[(1, 4)],
        &[(1, 2)],
        &[(1, 1)],
        &[(1, 1)],
        &[(1, 1)],
        &[(1, 1)],
        &[(1, 1)],
        &[(1, 1)],
      ],
    },
  );
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

/// Verifies the allocator metadata given a list of expected allocated blocks.
///
/// # Parameters
///
/// * `context` - The testing context.
/// * `tag` - A tag to include in check statements.
/// * `allocator` - The allocator to verify.
/// * `exp_alloc` - Expected allocation state.
fn verify_allocated_blocks(
  context: &mut test::TestContext,
  tag: &str,
  allocator: &PageAllocator,
  exp_alloc: AllocatorState,
) {
  for (level, exp) in iter::zip(&allocator.levels, exp_alloc.levels) {
    let mut idx = level.offset;
    let mut exp_idx = 0;
    let mut mask = 0x1;
    let mut avail = level.valid;

    for block in 0..level.valid {
      while exp_idx < exp.len() {
        let r = exp[exp_idx];
        assert!(r.0 > 0);
        assert!(r.1 >= r.0);

        if block > r.1 {
          exp_idx += 1;
        } else {
          break;
        }
      }

      let mut bit = mask;

      if exp_idx < exp.len() {
        let r = exp[exp_idx];

        if (block >= r.0 - 1) && (block <= r.1 - 1) {
          bit = 0;
          avail -= 1;
        }
      }

      check_eq!(context, allocator.flags[idx] & mask, bit, tag);

      mask <<= 1;

      if mask == 0 {
        idx += 1;
        mask = 0x1;
      }
    }

    check_eq!(context, level.avail, avail, tag);
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
