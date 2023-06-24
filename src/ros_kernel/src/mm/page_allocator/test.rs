use super::{BlockLevel, PageAllocator, WORD_SHIFT};
use crate::arch;
use crate::debug_print;
use crate::peripherals::memory;
use crate::support::range;
use crate::test;
use crate::test::macros::*;
use core::{iter, slice};

/// Test with 4 KiB pages.
const TEST_PAGE_SIZE: usize = 4096;

/// Test with 2047 pages. The non-power of 2 tests proper setup and accounting.
const TEST_MEM_SIZE: usize = TEST_PAGE_SIZE * 2047;

/// Each flag bit represents a pair of blocks. The number of blocks in a level
/// is `floor( pages / block size )`. The number of bits required at each level
/// is `ceil( blocks / 2 )`.
///
/// Block Size (Pages)       Bytes Required      Flag Bits
///                          32-bit  64-bit
/// -------------------------------------------------------
/// 1024                       4       8            1
///  512                       4       8            2
///  256                       4       8            4
///  128                       4       8            8
///   64                       4       8           16
///   32                       4       8           32
///   16                       8       8           64
///    8                      16      16          128
///    4                      32      32          256
///    2                      64      64          512
///    1                     128     128         1024
/// -------------------------------------------------------
///                          272     296
#[cfg(target_pointer_width = "32")]
const EXPECTED_METADATA_SIZE: usize = 272;

#[cfg(target_pointer_width = "64")]
const EXPECTED_METADATA_SIZE: usize = 296;

/// The allocator should serve up blocks of 2^0 up to 2^10 pages.
const EXPECTED_BLOCK_LEVELS: usize = 11;

/// Alignment type.
#[repr(align(0x400000))]
struct _Align4MiB;

/// Wrapper type to align the memory block. Aligning to 4 MiB allows the tests
/// to control how the allocator arranges blocks without needing to know the
/// kernel size.
struct _MemWrapper {
  _alignment: [_Align4MiB; 0],
  mem: [u8; TEST_MEM_SIZE + EXPECTED_METADATA_SIZE],
}

/// Use a statically allocated memory block within the kernel to avoid any
/// issues with memory configuration.
static mut TEST_MEM: _MemWrapper = _MemWrapper {
  _alignment: [],
  mem: [0; TEST_MEM_SIZE + EXPECTED_METADATA_SIZE],
};

/// Represents an allocator state usings lists of closed, 1-based ranges of
/// allocated blocks at each level.
struct AllocatorState<'a> {
  levels: [&'a [(usize, usize)]; EXPECTED_BLOCK_LEVELS],
}

pub fn run_tests() {
  execute_test!(test_size_calculation);
  execute_test!(test_level_construction);
  execute_test!(test_metadata_init);
  execute_test!(test_reservation_errors);
  execute_test!(test_reservations);
  execute_test!(test_allocation);
  execute_test!(test_free);
}

fn test_size_calculation(context: &mut test::TestContext) {
  let (_, size) = PageAllocator::make_levels(TEST_MEM_SIZE);
  check_eq!(context, size, EXPECTED_METADATA_SIZE);

  let (_, size) = PageAllocator::make_levels(0);
  check_eq!(context, size, 0);
}

fn test_level_construction(context: &mut test::TestContext) {
  let (levels, _) = PageAllocator::make_levels(TEST_MEM_SIZE);
  let exp_levels = make_expected_levels();

  check_eq!(context, levels.len(), exp_levels.len());

  for (a, b) in iter::zip(levels, exp_levels) {
    check_eq!(context, a.head, b.head);
    check_eq!(context, a.offset, b.offset);
  }
}

fn test_metadata_init(context: &mut test::TestContext) {
  let (mut allocator, avail) = make_allocator(0);
  allocator.init_metadata(&avail);
}

fn test_reservation_errors(context: &mut test::TestContext) {}

fn test_reservations(context: &mut test::TestContext) {}

fn test_allocation(context: &mut test::TestContext) {}

fn test_free(context: &mut test::TestContext) {}

#[cfg(target_pointer_width = "32")]
fn make_expected_levels() -> [BlockLevel; EXPECTED_BLOCK_LEVELS] {
  [
    BlockLevel { head: 0, offset: 0 },
    BlockLevel {
      head: 0,
      offset: 32,
    },
    BlockLevel {
      head: 0,
      offset: 48,
    },
    BlockLevel {
      head: 0,
      offset: 56,
    },
    BlockLevel {
      head: 0,
      offset: 60,
    },
    BlockLevel {
      head: 0,
      offset: 62,
    },
    BlockLevel {
      head: 0,
      offset: 63,
    },
    BlockLevel {
      head: 0,
      offset: 64,
    },
    BlockLevel {
      head: 0,
      offset: 65,
    },
    BlockLevel {
      head: 0,
      offset: 66,
    },
    BlockLevel {
      head: 0,
      offset: 67,
    },
  ]
}

#[cfg(target_pointer_width = "64")]
fn make_expected_levels() -> [BlockLevel; EXPECTED_BLOCK_LEVELS] {
  [
    BlockLevel { head: 0, offset: 0 },
    BlockLevel {
      head: 0,
      offset: 16,
    },
    BlockLevel {
      head: 0,
      offset: 24,
    },
    BlockLevel {
      head: 0,
      offset: 28,
    },
    BlockLevel {
      head: 0,
      offset: 30,
    },
    BlockLevel {
      head: 0,
      offset: 31,
    },
    BlockLevel {
      head: 0,
      offset: 32,
    },
    BlockLevel {
      head: 0,
      offset: 33,
    },
    BlockLevel {
      head: 0,
      offset: 34,
    },
    BlockLevel {
      head: 0,
      offset: 35,
    },
    BlockLevel {
      head: 0,
      offset: 36,
    },
  ]
}

fn make_allocator(base_offset: usize) -> (PageAllocator<'static>, memory::MemoryConfig) {
  let (levels, meta_size) = PageAllocator::make_levels(TEST_MEM_SIZE);
  let virt_addr = unsafe { TEST_MEM.mem.as_ptr() as usize };
  let base_addr = virt_addr - arch::get_kernel_virtual_base();
  let meta_addr = virt_addr + TEST_MEM_SIZE;
  let mut avail = memory::MemoryConfig::new();

  avail.insert_range(range::Range {
    base: base_addr + base_offset,
    size: TEST_MEM_SIZE - base_offset,
  });

  (
    PageAllocator {
      base: base_addr,
      size: TEST_MEM_SIZE + EXPECTED_METADATA_SIZE,
      levels,
      flags: unsafe { slice::from_raw_parts_mut(meta_addr as *mut usize, meta_size >> WORD_SHIFT) },
    },
    avail,
  )
}
