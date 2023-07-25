use super::{BlockLevel, INDEX_SHIFT, PageAllocator, WORD_BITS, WORD_MASK, WORD_SHIFT};
use crate::arch;
use crate::debug_print;
use crate::peripherals::memory;
use crate::support::range;
use crate::test;
use crate::test::macros::*;
use core::{iter, slice};

/// Test with 4 KiB pages.
const TEST_PAGE_SIZE: usize = 4096;
const TEST_PAGE_SHIFT: usize = 12;

/// Test with 2047 pages. The non-power of 2 tests proper setup and accounting.
const TEST_MEM_SIZE: usize = TEST_PAGE_SIZE * 2047;

/// Make the memory buffer larger to accommodate testing offset blocks.
const TEST_BUFFER_SIZE: usize = TEST_MEM_SIZE + (TEST_PAGE_SIZE * 256);

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

/// The total size of the test memory buffer.
const TOTAL_MEM_SIZE: usize = TEST_BUFFER_SIZE + EXPECTED_METADATA_SIZE;

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
  mem: [u8; TOTAL_MEM_SIZE],
}

/// Use a statically allocated memory block within the kernel to avoid any
/// issues with memory configuration.
static mut TEST_MEM: _MemWrapper = _MemWrapper {
  _alignment: [],
  mem: [0xcc; TOTAL_MEM_SIZE],
};

/// Represents an allocator state usings lists of block addresses.
struct AllocatorState<'a> {
  levels: [&'a [usize]; EXPECTED_BLOCK_LEVELS],
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
  let (virt_addr, _) = get_addrs();

  allocator.init_metadata(&avail);

  verify_allocator(context, &allocator, &AllocatorState {
    levels: [
      &[make_block_addr(virt_addr, 2047, 0)],
      &[make_block_addr(virt_addr, 1023, 1)],
      &[make_block_addr(virt_addr, 511, 2)],
      &[make_block_addr(virt_addr, 255, 3)],
      &[make_block_addr(virt_addr, 127, 4)],
      &[make_block_addr(virt_addr, 63, 5)],
      &[make_block_addr(virt_addr, 31, 6)],
      &[make_block_addr(virt_addr, 15, 7)],
      &[make_block_addr(virt_addr, 7, 8)],
      &[make_block_addr(virt_addr, 3, 9)],
      &[make_block_addr(virt_addr, 1, 10)],
    ]
  })
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

fn get_addrs() -> (usize, usize) {
  let virt_addr = unsafe { TEST_MEM.mem.as_ptr() as usize };
  let meta_addr = virt_addr + TEST_MEM_SIZE;

  (virt_addr, meta_addr)
}

fn make_block_addr(base_addr: usize, block: usize, level: usize) -> usize {
  assert!(block > 0);
  base_addr + ((TEST_PAGE_SIZE << level) * (block - 1))
}

fn make_allocator(base_offset: usize) -> (PageAllocator<'static>, memory::MemoryConfig) {
  let (levels, meta_size) = PageAllocator::make_levels(TOTAL_MEM_SIZE);
  let (virt_addr, meta_addr) = get_addrs();
  let base_addr = virt_addr - arch::get_kernel_virtual_base();

  unsafe { TEST_MEM.mem.fill(0xcc) };

  let mut avail = memory::MemoryConfig::new();

  avail.insert_range(range::Range {
    base: base_addr + base_offset,
    size: TEST_MEM_SIZE,
  });

  (
    PageAllocator {
      base: base_addr,
      size: TOTAL_MEM_SIZE,
      levels,
      flags: unsafe { slice::from_raw_parts_mut(meta_addr as *mut usize, meta_size >> WORD_SHIFT) },
    },
    avail,
  )
}

fn verify_allocator(
  context: &mut test::TestContext,
  allocator: &PageAllocator,
  state: &AllocatorState
) {
  let kernel_base = arch::get_kernel_virtual_base();
  let mut blocks = TEST_MEM_SIZE >> TEST_PAGE_SHIFT;
  let mut level_shift = 0;

  for (level, exp_blocks) in iter::zip(&allocator.levels, &state.levels) {
    if exp_blocks.is_empty() {
      check_eq!(context, level.head, 0);
      continue;
    }

    if level.head == 0 {
      mark_fail!(context, "Head pointer is null.");
      continue;
    }

    let mut ptr = level.head;
    let mut idx = 0;
    let mut mask = 0;

    let bits = (blocks + 1) >> 1;
    let words = (bits + WORD_BITS - 1) >> INDEX_SHIFT;
    blocks >>= 1;

    for block in *exp_blocks {
      let node = PageAllocator::get_block_node(ptr);
      check_eq!(context, ptr, *block);
      ptr = node.next;

      let page_num = ((*block - kernel_base) - allocator.base) >> TEST_PAGE_SHIFT;
      let block_num = page_num >> level_shift;
      let block_pair = (block_num + 1) >> 1;
      let block_idx = block_pair >> INDEX_SHIFT;

      if block_idx > idx {
        for i in idx..block_idx {
          check_eq!(context, allocator.flags[level.offset + i], mask);
          mask = 0;
        }

        idx = block_idx;
      }

      mask ^= 1 << (block_pair & WORD_MASK);
    }

    for i in idx..words {
      check_eq!(context, allocator.flags[level.offset + i], mask);
      mask = 0;
    }

    check_eq!(context, ptr, exp_blocks[0]);

    for block in exp_blocks.iter().rev() {
      let node = PageAllocator::get_block_node(ptr);
      ptr = node.prev;
      check_eq!(context, ptr, *block);
    }

    check_eq!(context, ptr, *exp_blocks.last().unwrap());

    level_shift += 1;
  }
}
