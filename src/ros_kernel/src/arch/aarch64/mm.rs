//! AArch64 Memory Management

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

/// Initialize memory.
///
/// # Parameters
///
/// * `virtual_base` - The kernel segment base address.
/// * `pages_start` - The address of the kernel's Level 1 page table.
/// * `pages_end` - The start of available memory for new pages.
/// * `mem_layout` - The physical memory layout.
///
/// # Description
///
/// Directly maps physical memory ranges into the kernel's virtual address
/// space.
///
/// # Returns
///
/// The new end of the page table area.
pub fn init(
  virtual_base: usize,
  pages_start: usize,
  pages_end: usize,
  mem_layout: &memory::MemoryConfig,
) -> usize {
  let mut pages_end = pages_end;

  for range in mem_layout.get_ranges() {
    pages_end = direct_map_memory(
      virtual_base,
      pages_start,
      pages_end,
      range.base,
      range.size,
      false,
    );
  }

  pages_end
}

/// Direct map a memory range into the kernel's virtual address space.
///
/// # Parameters
///
/// * `virtual_base` - The kernel segment base address.
/// * `pages_start` - The address of the kernel's Level 1 page table.
/// * `pages_end` - The start of available memory for new pages.
/// * `base` - Base of the physical address range.
/// * `size` - Size of the physical address range.
/// * `device` - Whether this block or page maps to device memory.
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
/// physical address space into the kernel segment. A physical address Ap maps
/// to the virtual address Av = virtual base + Ap.
///
/// This mapping is separate from allocating pages to the kernel.
///
/// # Returns
///
/// The new end of the page table area.
pub fn direct_map_memory(
  virtual_base: usize,
  pages_start: usize,
  pages_end: usize,
  base: usize,
  size: usize,
  device: bool,
) -> usize {
  fill_table(
    virtual_base,
    TableLevel::Level1,
    pages_start,
    pages_end,
    base,
    base,
    size,
    device,
  )
}

/// Map a range of physical addresses to the kernel's virtual address space.
/// This is a generalized version of `direct_map_memory` where `virt` != `base`.
/// A physical address Ap maps the the virtual address
/// Av = virtual base + (Ap - base + virt).
///
/// # Parameters
///
/// * `virtual_base` - The kernel segment base address.
/// * `pages_start` - The address of the kernel's Level 1 page table.
/// * `pages_end` - The start of available memory for new pages.
/// * `virt` - Base of the virtual address range.
/// * `base` - Base of the physical address range.
/// * `size` - Size of the physical address range.
/// * `device` - Whether this block or page maps to device memory.
///
/// # Returns
///
/// The new end of the page table area.
pub fn map_memory(
  virtual_base: usize,
  pages_start: usize,
  pages_end: usize,
  virt: usize,
  base: usize,
  size: usize,
  device: bool,
) -> usize {
  fill_table(
    virtual_base,
    TableLevel::Level1,
    pages_start,
    pages_end,
    virt,
    base,
    size,
    device,
  )
}

/// Given a table level, return the next table level down in the translation
/// hierarchy.
///
/// # Parameters
///
/// * `table_level` - The current table level.
///
/// # Returns
///
/// The next table level, or None if Level 4 is specified.
fn get_next_table(table_level: TableLevel) -> Option<TableLevel> {
  match table_level {
    TableLevel::Level1 => Some(TableLevel::Level2),
    TableLevel::Level2 => Some(TableLevel::Level3),
    TableLevel::Level3 => Some(TableLevel::Level4),
    TableLevel::Level4 => None,
  }
}

/// Given a table level, returns the size covered by a single entry.
///
/// # Parameters
///
/// * `table_level` - The table level of interest.
///
/// # Returns
///
/// The size covered by a single entry in bytes.
fn get_table_entry_size(table_level: TableLevel) -> usize {
  match table_level {
    TableLevel::Level1 => 1 << LEVEL_1_SHIFT,
    TableLevel::Level2 => 1 << LEVEL_2_SHIFT,
    TableLevel::Level3 => 1 << LEVEL_3_SHIFT,
    TableLevel::Level4 => 1 << LEVEL_4_SHIFT,
  }
}

/// Get the descriptor index for a virtual address in the specified table.
///
/// # Parameters
///
/// * `virt_addr` - The virtual address.
/// * `table_level` - The table level for the index.
///
/// # Description
///
/// With 4 KiB pages, the table indices are 9 bits each starting with Level 4 at
/// bit 12.
///
///     +---------+----+----+----+----+--------+
///     | / / / / | L1 | L2 | L3 | L4 | Offset |
///     +---------+----+----+----+----+--------+
///     63       47   39   30   21   12        0
///
/// # Returns
///
/// The index into the table at the specified level.
fn get_descriptor_index(virt_addr: usize, table_level: TableLevel) -> usize {
  match table_level {
    TableLevel::Level1 => (virt_addr >> LEVEL_1_SHIFT) & INDEX_MASK,
    TableLevel::Level2 => (virt_addr >> LEVEL_2_SHIFT) & INDEX_MASK,
    TableLevel::Level3 => (virt_addr >> LEVEL_3_SHIFT) & INDEX_MASK,
    TableLevel::Level4 => (virt_addr >> LEVEL_4_SHIFT) & INDEX_MASK,
  }
}

/// Check if a descriptor is valid. Bit 0 is the validity marker.
///
/// # Parameters
///
/// * `desc` - The descriptor.
///
/// # Returns
///
/// True if the descriptor is valid, false otherwise.
fn is_descriptor_valid(desc: usize) -> bool {
  (desc & 0x1) != 0
}

/// Get the physical address for either the next table or memory block from a
/// descriptor.
///
/// # Parameters
///
/// * `desc` - The descriptor.
///
/// # Returns
///
/// The physical address.
fn get_phys_addr_from_descriptor(desc: usize) -> usize {
  desc & ADDR_MASK
}

/// Create a table descriptor appropriate to the specified table level.
///
/// # Parameters
///
/// * `table_level` - The table level of the new entry.
/// * `phys_addr` - The physical address of the block or page.
/// * `device` - Whether this block or page maps to device memory.
///
/// # Description
///
/// The table level must be 2, 3, or 4. The Level 1 table can only point to
/// Level 2 tables.
///
/// # Returns
///
/// The new descriptor.
fn make_descriptor(table_level: TableLevel, phys_addr: usize, device: bool) -> usize {
  match table_level {
    TableLevel::Level2 | TableLevel::Level3 => make_block_descriptor(phys_addr, device),
    TableLevel::Level4 => make_page_descriptor(phys_addr, device),
    _ => {
      debug_assert!(false, "Invalid translation level.");
      0
    }
  }
}

/// Allocates a new page table if the specified descriptor is invalid, then
/// fills the table with entries for the specified range of memory.
///
/// # Parameters
///
/// * `virtual_base` - The kernel segment base address.
/// * `table_level` - The current table level.
/// * `desc` - The current descriptor in the table.
/// * `pages_end` - The current end of the table area.
/// * `virt` - Base of the virtual address range.
/// * `base` - Base of the physical address range.
/// * `size` - Size of the physical address range.
/// * `device` - Whether this block or page maps to device memory.
///
/// # Description
///
/// If the specified descriptor in the current table is invalid, a new page
/// is allocated at `pages_end` before a recursive call to `fill_table` is made.
///
/// The current table must be Level 1, 2, or 3. Level 4 tables can only point to
/// pages.
///
/// # Returns
///
/// The new end of the table area.
fn alloc_table_and_fill(
  virtual_base: usize,
  table_level: TableLevel,
  desc: usize,
  pages_end: usize,
  virt: usize,
  base: usize,
  size: usize,
  device: bool,
) -> (usize, usize) {
  let next_level = get_next_table(table_level).unwrap();
  let mut next_addr = get_phys_addr_from_descriptor(desc);
  let mut desc = desc;
  let mut pages_end = pages_end;

  if !is_descriptor_valid(desc) {
    next_addr = pages_end;
    pages_end += TABLE_SIZE;
    desc = make_pointer_entry(next_addr);
  }

  (
    desc,
    fill_table(
      virtual_base,
      next_level,
      next_addr,
      pages_end,
      virt,
      base,
      size,
      device,
    ),
  )
}

/// Map a block of physical memory.
///
/// # Parameters
///
/// * `phys_addr` - The physical address of the block.
/// * `device` - Whether this block maps to device memory.
///
/// # Returns
///
/// The new block descriptor.
fn make_block_descriptor(phys_addr: usize, device: bool) -> usize {
  let mut entry = (phys_addr & ADDR_MASK) | MM_ACCESS_FLAG | MM_BLOCK_FLAG;

  if device {
    entry |= MM_DEVICE_FLAG;
  }

  entry
}

/// Map a page of physical memory.
///
/// # Parameters
///
/// * `phys_addr` - The physical address of the page.
/// * `device` - Whether this block maps to device memory.
///
/// # Returns
///
/// The new page descriptor.
fn make_page_descriptor(phys_addr: usize, device: bool) -> usize {
  let mut entry = (phys_addr & ADDR_MASK) | MM_ACCESS_FLAG | MM_NORMAL_FLAG;

  if device {
    entry |= MM_DEVICE_FLAG;
  }

  entry
}

/// Make a pointer entry to a lower level page table.
///
/// # Parameters
///
/// * `phys_addr` - The physical address of the table.
///
/// # Returns
///
/// The new pointer entry.
fn make_pointer_entry(phys_addr: usize) -> usize {
  (phys_addr & ADDR_MASK) | MM_PAGE_TABLE_FLAG
}

/// Fills a page table with entries for the specified range.
///
/// # Parameters
///
/// * `virtual_base` - The kernel segment base address.
/// * `table_level` - The current table level.
/// * `table_addr` - The address of the current page table.
/// * `pages_end` - The start of available memory for new pages.
/// * `virt` - Base of the virtual address range.
/// * `base` - Base of the physical address range.
/// * `size` - Size of the physical address range.
/// * `device` - Whether this block or page maps to device memory.
///
/// # Details
///
/// The following cases need to be considered:
///
/// 1. The range size is greater than or equal to the entry size at this
///    translation level AND we are at a Level 2 or Level 3 table.
///
///    In this situation, we can create a block entry, then subtract the block
///    size from the total range size, loop around, and re-evaluate the
///    remaining size.
///
///    Sticking with 4 KiB pages and skipping Level 4 translation, a 128 GiB
///    address space would require 128 Level 3 tables, each with 512 2 MiB
///    entries, one Level 2 table, and the Level 1 table for a total of 520 KiB.
///    That can be reduced even more to 8 KiB and eliminate Level 3 translation
///    by using one Level 2 table with 128 1 GiB entries and one Level 1 table.
///
///    In practice, the ranges may not be all multiples of 1 GiB, so there will
///    be some mixture of Level 2, Level 3, and possibly Level 4 translation.
///
///    The goal is to keep the kernel page tables as compact as possible with as
///    few translation steps as necessary.
///
/// 2. The range size is greater than or equal to the entry size at this
///    translation level AND we are at the Level 1 or a Level 4 table.
///
///    In the Level 1 case, multiple Level 2 tables must be created but the
///    mechanics are otherwise the same.
///
///    The Level 4 case will just add page entries rather than block entries.
///
/// 3. The range size is less than the entry size at this translation level.
///
///    At Levels 1, 2, and 3, we need to first check if the current descriptor
///    is valid. If it is, we'll jump to the table at that address. If not,
///    we'll allocate a new table and increment the `pages_end` pointer.
///
///    With a valid table allocated, we'll recursively call `fill_table` with
///    the entry size divided by the number of table entries and the index shift
///    reduced by the number of bits in the entry count.
///
///    At Level 4, we will simply not map anything.
///
/// For example, using the typical 0x0 - 0x3c000000 range (960 MiB) on a 1 GiB
/// Raspberry Pi 3:
///
/// The first call starts at Level 1. The only choice is to jump to a Level 2
/// table, so we allocate a Level 2 table as necessary and jump to it.
///
/// The range is less than 1 GiB. So, we allocate a Level 3 table as necessary
/// and jump to it.
///
/// Now the range size is greater than or equal to the entry size. We can now
/// add blocks of 2 MiB to the Level 3 table until the remaining size is less
/// than the entry size.
///
/// 960 MiB is is a multiple of 2 MiB, so no Level 4 tables will be necessary.
///
/// # Returns
///
/// Returns the new end of the table area.
fn fill_table(
  virtual_base: usize,
  table_level: TableLevel,
  table_addr: usize,
  pages_end: usize,
  virt: usize,
  base: usize,
  size: usize,
  device: bool,
) -> usize {
  let entry_size = get_table_entry_size(table_level);
  let mut virt = virt;
  let mut base = base;
  let mut size = size;
  let mut pages_end = pages_end;
  let table = unsafe { &mut *((virtual_base + table_addr) as *mut PageTable) };

  loop {
    if size < PAGE_SIZE {
      break;
    }

    let idx = get_descriptor_index(virtual_base + virt, table_level);
    let mut fill_size = entry_size;

    if size < entry_size || table_level == TableLevel::Level1 {
      // Case 2 with a Level 1 table and Case 3 are basically the same, we just
      // need to make sure to take the minimum of the block size and the entry
      // size since the block size can be greater at Level 1.
      fill_size = cmp::min(size, entry_size);

      (table.entries[idx], pages_end) = alloc_table_and_fill(
        virtual_base,
        table_level,
        table.entries[idx],
        pages_end,
        virt,
        base,
        fill_size,
        device,
      );
    } else {
      // Handle Case 1 and Case 2 for Level 4 tables.
      table.entries[idx] = make_descriptor(table_level, base, device);
    }

    virt += fill_size;
    base += fill_size;
    size -= fill_size;
  }

  // Return the updated `pages_end` pointer to be used by subsequent mappings.
  pages_end
}
