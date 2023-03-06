//! AArch64 memory management.

use crate::peripherals::memory;
use core::cmp;

const TABLE_SIZE: usize = 4096;
const PAGE_SHIFT: usize = 12;
const PAGE_SIZE: usize = 1 << PAGE_SHIFT;
const PAGE_MASK: usize = PAGE_SIZE - 1;
const INDEX_SHIFT: usize = 9;
const INDEX_SIZE: usize = 1 << INDEX_SHIFT;
const INDEX_MASK: usize = INDEX_SIZE - 1;
const LEVEL_1_SHIFT: usize = PAGE_SHIFT + (3 * INDEX_SHIFT);
const LEVEL_2_SHIFT: usize = PAGE_SHIFT + (2 * INDEX_SHIFT);
const LEVEL_3_SHIFT: usize = PAGE_SHIFT + INDEX_SHIFT;
const LEVEL_4_SHIFT: usize = PAGE_SHIFT;
const ADDR_MASK: usize = ((1 << 48) - 1) & !PAGE_MASK;
const MM_PAGE_TABLE_FLAG: usize = 0x3 << 0;
const MM_BLOCK_FLAG: usize = 0x1 << 0;
const MM_NORMAL_FLAG: usize = 0x1 << 2;
const MM_DEVICE_FLAG: usize = 0x0 << 2;
const _MM_RO_FLAG: usize = 0x10 << 6;
const MM_ACCESS_FLAG: usize = 0x1 << 10;

/// Translation table level.
#[derive(Clone, Copy, PartialEq)]
enum TableLevel {
  Level1,
  Level2,
  Level3,
  Level4,
}

/// Page table structure for 4 KiB pages.
#[repr(C)]
struct PageTable {
  entries: [usize; 512],
}

/// Initialize the AArch64 page tables for the kernel.
///
/// # Parameters
///
/// * `virtual_base` - The kernel segment base address.
/// * `pages_start` - The address of the kernel's Level 1 page table.
/// * `mem_config` - The memory layout.
///
/// # Details
///
///     TODO: For now, memory management will just assume 4 KiB pages. The
///           bootstrap code will have already configured TCR_EL1 with 4 KiB
///           granules.
///
/// The canonical 64-bit virtual address space layout for a process looks like:
///
///     +-----------------+ 0xffff_ffff_ffff_ffff
///     |                 |
///     | Kernel Segment  | 128 TiB
///     |                 |
///     +-----------------+ 0xffff_8000_0000_0000
///     | / / / / / / / / |
///     | / / / / / / / / | 16,776,960 TiB of unused address space
///     | / / / / / / / / |
///     +-----------------+ 0x0000_8000_0000_0000
///     |                 |
///     | User Segment    | 128 TiB
///     |                 |
///     +-----------------+ 0x0000_0000_0000_0000
///
/// AArch64 provides two independent registers for address translation so that
/// the kernel does not need to be mapped into the translation tables for every
/// process. The most-significant bit selects the register used for translation.
///
/// AArch64 provides four levels of address space translation. With 4 KiB pages,
/// the page tables can address 256 TiB of memory:
///
///     Level 1   ->    Level 2   ->    Level 3   ->    Level 4
///     Covers          Covers          Covers          Covers
///     256 TiB         512 GiB         1 GiB           2 MiB
///
/// Each page table itself is 4 KiB (512 entries, each 64-bits).
///
/// AArch64 allows skipping lower levels of translation. Each Level 2 entry can
/// point to a Level 3 table OR a 1 GiB block of memory. Each Level 3 entry can
/// point to a Level 4 table OR a 2 MiB block of memory.
///
/// Currently, a single kernel is not expected to deal with anywhere near 128
/// TiB of physical memory, so it is feasible to directly map the entire
/// physical address space into the kernel segment.
///
/// This mapping is separate from allocating pages to the kernel.
pub fn init_memory(virtual_base: usize, pages_start: usize, mem_config: &memory::MemoryConfig) {
  // The bootstrap code set up Level 1, 2, and 3 tables for the initial mapping.
  // We'll just reuse those and add new tables after.
  let mut pages_end = pages_start + (3 * PAGE_SIZE);

  for range in mem_config.get_ranges() {
    pages_end = fill_table(
      virtual_base,
      TableLevel::Level1,
      pages_start,
      pages_end,
      range,
    );
  }
}

fn get_next_table(table_level: TableLevel) -> Option<TableLevel> {
  match table_level {
    TableLevel::Level1 => Some(TableLevel::Level2),
    TableLevel::Level2 => Some(TableLevel::Level3),
    TableLevel::Level3 => Some(TableLevel::Level4),
    TableLevel::Level4 => None,
  }
}

fn get_table_entry_size(table_level: TableLevel) -> usize {
  match table_level {
    TableLevel::Level1 => 1 << LEVEL_1_SHIFT,
    TableLevel::Level2 => 1 << LEVEL_2_SHIFT,
    TableLevel::Level3 => 1 << LEVEL_3_SHIFT,
    TableLevel::Level4 => 1 << LEVEL_4_SHIFT,
  }
}

/// Get the descriptor index for a virtual address in the specified table.
fn get_descriptor_index(virt_addr: usize, table_level: TableLevel) -> usize {
  match table_level {
    TableLevel::Level1 => (virt_addr >> LEVEL_1_SHIFT) & INDEX_MASK,
    TableLevel::Level2 => (virt_addr >> LEVEL_2_SHIFT) & INDEX_MASK,
    TableLevel::Level3 => (virt_addr >> LEVEL_3_SHIFT) & INDEX_MASK,
    TableLevel::Level4 => (virt_addr >> LEVEL_4_SHIFT) & INDEX_MASK,
  }
}

/// Check if a descriptor is valid. Bit 0 is the validity marker.
fn is_descriptor_valid(desc: usize) -> bool {
  (desc & 0x1) != 0
}

/// Get the physical address for either the next table or memory block from a
/// descriptor.
fn get_phys_addr_from_descriptor(desc: usize) -> usize {
  desc & ADDR_MASK
}

/// Make a normal memory block that is R/W for the kernel.
fn make_normal_block_entry(phys_addr: usize) -> usize {
  (phys_addr & ADDR_MASK) | MM_ACCESS_FLAG | MM_BLOCK_FLAG | MM_NORMAL_FLAG
}

/// Make a device memory block that is R/W for the kernel.
fn make_device_block_entry(phys_addr: usize) -> usize {
  (phys_addr & ADDR_MASK) | MM_ACCESS_FLAG | MM_BLOCK_FLAG | MM_DEVICE_FLAG
}

fn make_normal_page_entry(phys_addr: usize) -> usize {
  (phys_addr & ADDR_MASK) | MM_ACCESS_FLAG | MM_NORMAL_FLAG;
}

fn make_device_page_entry(phys_addr: usize) -> usize {
  (phys_addr & ADDR_MASK) | MM_ACCESS_FLAG | MM_DEVICE_FLAG;
}

/// Make a pointer entry to a lower level page table.
fn make_pointer_entry(phys_addr: usize) -> usize {
  (phys_addr & ADDR_MASK) | MM_PAGE_TABLE_FLAG
}

/// Fills a page table with entries for the specified range.
///
/// # Details
///
/// There are two cases to consider here:
///
/// 1. The range size is greater than or equal to the entry size at this
///    translation level AND we are at a Level 2 or Level 3 table.
///
///    In this situation, we can create a block entry, then subtract the block
///    size from the total range size, loop around, and re-evaluate the
///    remaining size.
///
///    Sticking with 4 KiB pages and skipping Level 4 translation, a 128 GiB
///    address space would require 128 Level 3 tables, one Level 2 table, and
///    the Level 1 table for a total of 520 KiB. That can be reduced to 8 KiB
///    and eliminate Level 3 translation by using one Level 2 table with 128 
///    1 GiB entries and one Level 1 table.
///
///    In practice, the ranges may not be all multiples of 1 GiB, so there will
///    be some mixture of Level 2 and Level 3 translation.
///
/// 2. The range size is less than the entry size at this translation level.
///
///    In this situation, we need to first check if the current descriptor is
///    valid. If it is, we'll jump to the table at that address. If not, we'll
///    allocate a new table and increment the `pages_end` pointer.
///
///    With a valid table allocated, we'll recursively call `fill_table` with
///    the entry size divided by the number of table entries and the index shift
///    reduced by the number of bits in the entry count.
///
/// For example, using the typical 0x0 - 0x3c000000 range on a 1 GiB Raspberry
/// Pi 3:
///
/// The first call should have an entry size of 512 GiB and an index shift of 39
/// bits. The range size is less than 512 GiB, so we jump to a Level 2 table,
/// and recursively call with an entry size of 1 GiB and an index shift of 30
/// bits.
///
/// The range size is, again, less than 1 GiB. So, we jump to a Level 3 table,
/// and recursively call with an entry size of 2 MiB and an index shift of 21
/// bits.
///
/// Now the range size is greater than or equal to the entry size. We can now
/// add blocks of 2 MiB to the Level 3 table until the remaining size is less
/// than the entry size. If we needed to jump to a Level 4 table to handle the
/// remainder with 4 KiB pages, we could.
fn fill_table(
  virtual_base: usize,
  table_level: TableLevel,
  table_addr: usize,
  pages_end: usize,
  range: &memory::MemoryRange,
) -> usize {
  let entry_size = get_table_entry_size(table_level);
  let mut base = range.base;
  let mut size = range.size;
  let mut pages_end = pages_end;
  let table = unsafe { &mut *((virtual_base + table_addr) as *mut PageTable) };

  loop {
    let idx = get_descriptor_index(virtual_base + base, table_level);

    if size >= entry_size
      && (table_level == TableLevel::Level2 || table_level == TableLevel::Level3)
    {
      // If possible, create a block entry. We cannot create block entries at
      // Level 1 or Level 4.
      if range.device {
        table.entries[idx] = make_device_block_entry(base);
      } else {
        table.entries[idx] = make_normal_block_entry(base);
      }

      base += entry_size;
      size -= entry_size;
    } else if size > 0 {
      // Move to the next table down.
      let next_level = get_next_table(table_level).expect("Invalid table level.");
      let desc = table.entries[idx];
      let mut next_addr = get_phys_addr_from_descriptor(desc);

      if !is_descriptor_valid(desc) {
        next_addr = pages_end;
        pages_end += TABLE_SIZE;
        table.entries[idx] = make_pointer_entry(base);
      }

      let fill_size = cmp::min(size, entry_size);
      let fill = memory::MemoryRange {
        base,
        size: fill_size,
        device: range.device,
      };

      pages_end = fill_table(virtual_base, next_level, next_addr, pages_end, &fill);

    }

    if size < entry_size {
      break;
    }
  }

  // Return the updated `pages_end` pointer up the call stack to be used by
  // subsequent mappings.
  pages_end
}
