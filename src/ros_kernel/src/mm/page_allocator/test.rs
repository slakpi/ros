use super::{PageAllocator, PageLevel, INDEX_SHIFT, WORD_LEN, WORD_SIZE};
use crate::debug_print;
use crate::test;
use crate::test::macros::*;
use core::{iter, slice};

/// Represents an allocator state usings lists of closed, 1-based ranges of
/// allocated blocks at each level.
struct AllocatorState<'a> {
  levels: [&'a [(usize, usize)]; EXPECTED_PAGE_LEVELS],
}

/// Test with 4 KiB pages.
const TEST_PAGE_SIZE: usize = 4096;

/// Test with 2047 pages. The non-power of 2 tests proper setup and accounting.
const TEST_MEM_SIZE: usize = TEST_PAGE_SIZE * 2047;

/// Block Size (Pages)       Bytes Required      Valid Bits
///                          32-bit  64-bit
/// -------------------------------------------------------
/// 1024                       4       8            1
///  512                       4       8            3
///  256                       4       8            7
///  128                       4       8           15
///   64                       4       8           31
///   32                       8       8           63
///   16                      16      16          127
///    8                      32      32          255
///    4                      64      64          511
///    2                     128     128         1023
///    1                     256     256         2047
/// -------------------------------------------------------
///                          524     544
#[cfg(target_pointer_width = "32")]
const EXPECTED_METADATA_SIZE: usize = 524;

#[cfg(target_pointer_width = "64")]
const EXPECTED_METADATA_SIZE: usize = 544;

/// The allocator should serve up blocks of 2^0 up to 2^10 pages.
const EXPECTED_PAGE_LEVELS: usize = 11;

pub fn run_tests() {
  execute_test!(test_size_calculation);
  execute_test!(test_level_construction);
  execute_test!(test_metadata_init);
  execute_test!(test_reservation_errors);
  execute_test!(test_reservations);
  execute_test!(test_allocation);
}

fn test_size_calculation(context: &mut test::TestContext) {
  let size = PageAllocator::calc_metadata_size(TEST_MEM_SIZE);
  check_eq!(context, size, EXPECTED_METADATA_SIZE);
}

fn test_level_construction(context: &mut test::TestContext) {
  let (levels, _) = PageAllocator::make_levels(TEST_MEM_SIZE);
  let exp_levels = make_expected_levels();
  check_eq!(context, levels.len(), exp_levels.len());

  for (a, b) in iter::zip(levels, exp_levels) {
    check_eq!(context, a.offset, b.offset);
    check_eq!(context, a.valid, b.valid);

    // `make_levels` does not determine the number of available blocks at each
    // level. That is done when initializing the allocator's metadata.
    check_eq!(context, a.avail, 0);
  }
}

fn test_metadata_init(context: &mut test::TestContext) {
  #[cfg(target_pointer_width = "32")]
  const LAST_WORDS: [usize; EXPECTED_PAGE_LEVELS] = [
    0x7fffffff, 0x7fffffff, 0x7fffffff, 0x7fffffff, 0x7fffffff, 0x7fffffff, 0x7fffffff, 0x00007fff,
    0x0000007f, 0x00000007, 0x00000001,
  ];

  #[cfg(target_pointer_width = "64")]
  const LAST_WORDS: [usize; EXPECTED_PAGE_LEVELS] = [
    0x7fffffffffffffff,
    0x7fffffffffffffff,
    0x7fffffffffffffff,
    0x7fffffffffffffff,
    0x7fffffffffffffff,
    0x7fffffffffffffff,
    0x000000007fffffff,
    0x0000000000007fff,
    0x000000000000007f,
    0x0000000000000007,
    0x0000000000000001,
  ];

  let exp_levels = make_expected_levels();

  // Initialize all bytes in the metadata buffer to 0 to ensure flag
  // initialization sets them appropriately. No bytes should be zero after
  // initialization.
  let mut buffer: [u8; EXPECTED_METADATA_SIZE] = [0; EXPECTED_METADATA_SIZE];

  // Get a page allocator.
  let mut allocator = make_allocator(0x2000, buffer.as_mut_ptr());

  // Test flag initialization.
  allocator.init_metadata();

  // Verify the availability counts match.
  for (a, b) in iter::zip(&allocator.levels, exp_levels) {
    check_eq!(context, a.avail, b.avail);
    check_eq!(context, a.avail, a.valid);
  }

  // Verify the availability flags.
  for (level, last_word) in iter::zip(&allocator.levels, &LAST_WORDS) {
    let last = level.valid >> INDEX_SHIFT;

    for i in 0..last {
      check_eq!(context, allocator.flags[level.offset + i], usize::MAX);
    }

    check_eq!(context, allocator.flags[level.offset + last], *last_word);
  }
}

fn test_reservation_errors(context: &mut test::TestContext) {
  const TEST_ALLOC_BASE: usize = 0x8000;

  let mut a_buff: [u8; EXPECTED_METADATA_SIZE] = [0xff; EXPECTED_METADATA_SIZE];
  let mut b_buff: [u8; EXPECTED_METADATA_SIZE] = [0xfe; EXPECTED_METADATA_SIZE];
  let mut a = make_allocator(TEST_ALLOC_BASE, a_buff.as_mut_ptr());
  let mut b = make_allocator(TEST_ALLOC_BASE, b_buff.as_mut_ptr());

  a.init_metadata();
  b.init_metadata();

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

  allocator.init_metadata();

  // Reserve a 0xe000 byte block starting at 0x2001. Page-alignment forces
  // reservation of 0x2000 - 0x10fff (Pages 3 - 17).
  allocator.reserve(TEST_ALLOC_BASE + 0x2001, 0xe000);
  verify_allocated_blocks(
    context,
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

fn test_allocation(context: &mut test::TestContext) {
  const TEST_ALLOC_BASE: usize = 0x400000;
  const TEST_ALLOC_END: usize = TEST_ALLOC_BASE + TEST_MEM_SIZE;

  let mut buffer: [u8; EXPECTED_METADATA_SIZE] = [0; EXPECTED_METADATA_SIZE];
  let mut allocator = make_allocator(TEST_ALLOC_BASE, buffer.as_mut_ptr());

  // Test allocating blocks of every size and verify the base address returned
  // is within the memory served by the allocator.
  for level in 0..EXPECTED_PAGE_LEVELS {
    allocator.init_metadata();

    let pages = 1 << level;
    let size = pages * TEST_PAGE_SIZE;

    if let Some(addr) = allocator.allocate(pages) {
      check_gteq!(context, addr, TEST_ALLOC_BASE);
      check_lteq!(context, addr + size, TEST_ALLOC_END);
    } else {
      mark_fail!(context, "Allocation failed.");
    }
  }

  // Test allocating all blocks in a level, then verifying the available blocks
  // blocks in each level below the current level.
  //
  //   NOTE: For this test to be meaningful, the number of pages should not be a
  //         power of 2.
  for level in 1..EXPECTED_PAGE_LEVELS {
    allocator.init_metadata();

    let pages = 1 << level;
    let total_pages = pages * allocator.levels[level].valid;

    for _ in 0..allocator.levels[level].valid {
      _ = allocator.allocate(pages);
    }

    for i in 0..level {
      let blocks = total_pages >> i;
      check_eq!(
        context,
        allocator.levels[i].avail,
        allocator.levels[i].valid - blocks
      );
    }
  }

  // Test attempting to allocating too many blocks of every size.
  for level in 0..EXPECTED_PAGE_LEVELS {
    allocator.init_metadata();

    let pages = 1 << level;

    for _ in 0..allocator.levels[level].valid {
      check_eq!(context, allocator.allocate(pages).is_some(), true);
    }

    check_eq!(context, allocator.allocate(pages).is_none(), true);
  }

  // Test attempting to allocate a block that is larger than the maximum size.
  let max_pages = 1 << (EXPECTED_PAGE_LEVELS - 1);
  allocator.init_metadata();
  check_eq!(context, allocator.allocate(max_pages + 1).is_none(), true);

  // Test attempting to allocate zero pages.
  allocator.init_metadata();
  check_eq!(context, allocator.allocate(0).is_none(), true);
}

fn make_expected_levels() -> [PageLevel; EXPECTED_PAGE_LEVELS] {
  let mut levels: [PageLevel; EXPECTED_PAGE_LEVELS] = [
    PageLevel {
      offset: 0,
      valid: 2047,
      avail: 2047,
    },
    PageLevel {
      offset: 0,
      valid: 1023,
      avail: 1023,
    },
    PageLevel {
      offset: 0,
      valid: 511,
      avail: 511,
    },
    PageLevel {
      offset: 0,
      valid: 255,
      avail: 255,
    },
    PageLevel {
      offset: 0,
      valid: 127,
      avail: 127,
    },
    PageLevel {
      offset: 0,
      valid: 63,
      avail: 63,
    },
    PageLevel {
      offset: 0,
      valid: 31,
      avail: 31,
    },
    PageLevel {
      offset: 0,
      valid: 15,
      avail: 15,
    },
    PageLevel {
      offset: 0,
      valid: 7,
      avail: 7,
    },
    PageLevel {
      offset: 0,
      valid: 3,
      avail: 3,
    },
    PageLevel {
      offset: 0,
      valid: 1,
      avail: 1,
    },
  ];

  for i in 1..EXPECTED_PAGE_LEVELS {
    levels[i].offset = levels[i - 1].offset + ((levels[i - 1].valid + WORD_LEN - 1) / WORD_LEN);
  }

  levels
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
  let (levels, alloc_size) = PageAllocator::make_levels(TEST_MEM_SIZE);
  let words = alloc_size / WORD_SIZE;

  // Manually create a PageAllocator to prevent it from performing any flag
  // initializations or memory reservations.
  PageAllocator {
    base,
    size: TEST_MEM_SIZE,
    flags: unsafe { slice::from_raw_parts_mut(mem as *mut usize, words) },
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

      check_eq!(context, allocator.flags[idx] & mask, bit);

      mask <<= 1;

      if mask == 0 {
        idx += 1;
        mask = 0x1;
      }
    }

    check_eq!(context, level.avail, avail);
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

    let last = a.offset + (a.valid >> INDEX_SHIFT);

    if act.flags[a.offset..=last] != exp.flags[e.offset..=last] {
      return false;
    }
  }

  true
}
