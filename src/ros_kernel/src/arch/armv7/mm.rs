//! ARMv7a Memory Management

use super::task;
use core::{cmp, ptr, slice};

const PAGE_SHIFT: usize = 12;
const PAGE_SIZE: usize = 1 << PAGE_SHIFT;
const PAGE_MASK: usize = PAGE_SIZE - 1;

const LEVEL_1_SHIFT_LONG: usize = 30;
const LEVEL_2_SHIFT_LONG: usize = 21;
const LEVEL_3_SHIFT_LONG: usize = 12;
const INDEX_SHIFT_LONG: usize = 9;
const INDEX_MASK_LONG: usize = (1 << INDEX_SHIFT_LONG) - 1;

/// With LPAE, the Level 1 table only has 4 entries, but let it use an entire
/// 4 KiB page so that it matches the Level 2 and 3 table sizes.
const TABLE_SIZE_LONG: usize = 512 * 8;

const ADDR_MASK_LONG: usize = 0xffff_f000;
const MM_PAGE_TABLE_FLAG_LONG: usize = 0x3 << 0;
const MM_BLOCK_FLAG_LONG: usize = 0x1 << 0;
const MM_PAGE_FLAG_LONG: usize = 0x3 << 0;
const MM_ACCESS_FLAG_LONG: usize = 0x1 << 10;
const MM_NORMAL_MAIR_IDX_LONG: usize = 0x0 << 2;
const MM_DEVICE_MAIR_IDX_LONG: usize = 0x1 << 2;

const TYPE_MASK: usize = 0x3;

/// Physical start address of the high memory area.
const HIGH_MEMORY: usize = 0x3800_0000;

/// Translation table level. LPAE supports up to 3 levels of translation.
#[derive(Clone, Copy, PartialEq)]
enum TableLevel {
  Level1,
  Level2,
  Level3,
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
  let virt = virtual_base + base;

  fill_table(
    virtual_base,
    get_first_table(virtual_base, virt),
    pages_start,
    pages_end,
    virt,
    base,
    size,
    device,
  )
}

/// Map a range of physical addresses to a task's virtual address space.
///
/// # Parameters
///
/// * `virtual_base` - The kernel segment base address.
/// * `pages_start` - The address of the task's Level 1 page table.
/// * `pages_end` - The start of available memory for new page tables.
/// * `virt` - Base of the virtual address range.
/// * `base` - Base of the physical address range.
/// * `size` - Size of the physical address range.
/// * `device` - Whether this block or page maps to device memory.
///
/// # Description
///
/// This is a generalized version of `direct_map_memory` where `virt` != `base`.
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
    get_first_table(virtual_base, virt),
    pages_start,
    pages_end,
    virt,
    base,
    size,
    device,
  )
}

/// Allocates a new page table if necessary, then fills the table with entries
/// for the specified range of memory.
///
/// # Parameters
///
/// * `virtual_base` - The kernel segment base address.
/// * `table_level` - The current table level.
/// * `desc` - The current descriptor in the table.
/// * `desc_high` - High 32-bits of a long descriptor (0 if LPAE not supported).
/// * `pages_end` - The current end of the table area.
/// * `virt` - Base of the virtual address range.
/// * `base` - Base of the physical address range.
/// * `size` - Size of the physical address range.
/// * `device` - Whether this block or page maps to device memory.
///
/// # Returns
///
/// The new descriptor and new end of the table area.
fn alloc_table_and_fill(
  virtual_base: usize,
  table_level: TableLevel,
  desc: usize,
  desc_high: usize,
  pages_end: usize,
  virt: usize,
  base: usize,
  size: usize,
  device: bool,
) -> (usize, usize, usize) {
  let next_level = get_next_table(table_level).unwrap();
  let mut next_addr = get_phys_addr_from_descriptor(desc, desc_high);
  let mut desc = desc;
  let mut desc_high = desc_high;
  let mut pages_end = pages_end;

  // TODO: It is probably fine to overwrite a section descriptor. If the memory
  //       configuration is overwriting itself, then we probably have something
  //       wrong and a memory trap is the right outcome.
  if !is_pointer_entry(desc, desc_high) {
    let table_size = get_table_size(table_level);
    next_addr = pages_end;
    pages_end += table_size;

    unsafe {
      // Zero out the table. Any entry in the table with bits 0 and 1 set to 0
      // is invalid.
      ptr::write_bytes((virtual_base + next_addr) as *mut u8, 0, table_size);
    }

    (desc, desc_high) = make_pointer_entry(next_addr);
  }

  (
    desc,
    desc_high,
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

/// Get the first table level to translate a given virtual address.
///
/// # Parameters
///
/// * `virtual_base` - The kernel segment base address.
/// * `virt_addr` - The virtual address.
///
/// # Description
///
///   NOTE: Assumes 3/1 or 2/2 split.
///
/// If `virt_addr` is not in the kernel segment, start from a Level 1 table.
/// If `virt_addr` is in the kernel segment and a 3/1 split is used, start from
/// a Level 2 table. Otherwise, start from a Level 1 table.
///
/// # Returns
///
/// The starting table level.
fn get_first_table(virtual_base: usize, virt_addr: usize) -> TableLevel {
  if virt_addr & virtual_base == virtual_base {
    if 0xffff_ffff - virtual_base < 0x4000_0000 {
      TableLevel::Level2
    } else {
      TableLevel::Level1
    }
  } else {
    TableLevel::Level1
  }
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
/// The next table level. None if Level 2 is specified (no LPAE) or Level 3 is
/// is specified (with LPAE).
fn get_next_table(table_level: TableLevel) -> Option<TableLevel> {
  match table_level {
    TableLevel::Level1 => Some(TableLevel::Level2),
    TableLevel::Level2 => Some(TableLevel::Level3),
    TableLevel::Level3 => None,
  }
}

/// Get the physical address for either the next table from a descriptor.
///
/// # Parameters
///
/// * `desc` - The descriptor.
/// * `desc_high` - High 32-bits of a long descriptor (0 if LPAE not supported).
///
/// # Description
///
///   NOTE: Does not support LPAE 40-bit pointers. The high 32-bits of the
///         descriptor are ignored.
///
/// # Returns
///
/// The physical address.
fn get_phys_addr_from_descriptor(desc: usize, _desc_high: usize) -> usize {
  desc & ADDR_MASK_LONG
}

/// Given a table level, determine the size of the table.
///
/// # Parameters
///
/// * `table_level` - The current table level.
///
/// # Returns
///
/// The size of the table in bytes.
fn get_table_size(table_level: TableLevel) -> usize {
  TABLE_SIZE_LONG
}

/// Determine if a descriptor is a table pointer entry.
///
/// # Parameters
///
/// * `desc` - The current descriptor in the table.
/// * `desc_high` - High 32-bits of a long descriptor (0 if LPAE not supported).
///
/// # Returns
///
/// True if the descriptor is a page table pointer, false otherwise.
fn is_pointer_entry(desc: usize, _desc_high: usize) -> bool {
  desc & TYPE_MASK == MM_PAGE_TABLE_FLAG_LONG
}

/// Make a pointer entry to a lower level page table.
///
/// # Parameters
///
/// * `phys_addr` - The physical address of the table.
///
/// # Returns
///
/// A tuple with the low and high 32-bits of the descriptor. The high 32-bits
/// are zero if LPAE is not supported.
fn make_pointer_entry(phys_addr: usize) -> (usize, usize) {
  ((phys_addr & ADDR_MASK_LONG) | MM_PAGE_TABLE_FLAG_LONG, 0)
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
///     TODO: For now, memory management will just assume 4 KiB pages. The start
///           code will have already configured the MMU and provided the page
///           size in the kernel configuration struct.
///
/// ARMv7a provides two independent registers for address translation so that
/// the kernel does not need to be mapped into the translation tables for every
/// process. The most-significant bit selects the register used for translation.
///
/// The "classic" ARM MMU supports two levels of address translation using
/// 32-bit page table descriptors.
///
///     Level 1       ->  Level 2       
///     4096 Entries      256 Entries
///     Covers 4 GiB      Covers 1 MiB
///
/// With short page table descriptors, if the address space is split between
/// user space and kernel space, the user address space cannot be larger than
/// 2 GiB (even 2/2 split).
///
/// When an ARMv7a CPU implements the Large Physical Address Extensions, it
/// supports the long page table descriptor format. Instead of the "classic"
/// two-level translation tables, the MMU supports three levels of address
/// translation using 64-bit page table descriptors.
///
///     Level 1       ->  Level 2       -> Level 3
///     4 Entries         512 Entries      512 Entries
///     Covers 4 GiB      Covers 1 GiB     Covers 2 MiB
///
/// Additionally, LPAE allows configuring the MMU to increase the size of the
/// user address space making a 3/1 split possible.
///
///     NOTE: The MMU will AUTOMATICALLY skip Level 1 translation if the size of
///           a segment is 1 GiB or less. In a 3/1 split, the MMU expects that
///           TTBR1 points directly to the kernel segment's Level 2 table.
///
/// Refer to the AArch64 version of `fill_table`. The table arrangement is
/// different with/out LPAE, but the logic is the same.
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

  while size >= PAGE_SIZE {
    let idx = get_descriptor_index(virtual_base, virt, table_level);
    let table = get_table(virtual_base + table_addr);
    let mut fill_size = entry_size;
    let desc: usize;
    let desc_high: usize;

    if size < entry_size || table_level == TableLevel::Level1 {
      fill_size = cmp::min(size, entry_size);

      (desc, desc_high, pages_end) = alloc_table_and_fill(
        virtual_base,
        table_level,
        table[idx],
        table[idx + 1],
        pages_end,
        virt,
        base,
        fill_size,
        device,
      );
    } else {
      (desc, desc_high) = make_descriptor(table_level, base, device).unwrap();
    }

    table[idx] = desc;
    table[idx + 1] = desc_high;

    virt += fill_size;
    base += fill_size;
    size -= fill_size;
  }

  // Return the updated `pages_end` pointer to be used by subsequent mappings.
  pages_end
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
    TableLevel::Level1 => 1 << LEVEL_1_SHIFT_LONG,
    TableLevel::Level2 => 1 << LEVEL_2_SHIFT_LONG,
    TableLevel::Level3 => PAGE_SIZE,
  }
}

/// Get a memory slice for the table at a given address.
///
/// # Parameters
///
/// * `table_addr` - The table address.
///
/// # Description
///
///   NOTE: Expects all tables to be TABLE_SIZE_LONG including Level 1 tables.
///
/// # Returns
///
/// A slice of the correct size for the table level.
fn get_table(table_addr: usize) -> &'static mut [usize] {
  unsafe {
    // Note the shift right by 2 instead of 3. The slice is 32 bits, not 64.
    slice::from_raw_parts_mut(table_addr as *mut usize, TABLE_SIZE_LONG >> 2)
  }
}

/// Get the descriptor index for a virtual address in the specified table.
///
/// # Parameters
///
/// * `virtual_base` - The kernel segment base address.
/// * `virt_addr` - The virtual address.
/// * `table_level` - The table level for the index.
///
/// # Description
///
/// Without LPAE:
///
///   +-+----------+-------+-----------+
///   |/|    L1    |  L2   |  Offset   |
///   +-+----------+-------+-----------+
///    31         20      12           0
///
/// With LPAE:
///
///   +----+--------+--------+-----------+
///   | L1 |   L2   |   L3   |  Offset   |
///   +----+--------+--------+-----------+
///   31  30       21       12           0
///
///   NOTE: The index is in 32-bit words. When using LPAE, the index returned
///         by this function, `N`, is the low 32-bits of the descriptor while
///         the index `N + 1` is the high 32-bits.
///
/// # Returns
///
/// The index into the table at the specified level.
fn get_descriptor_index(virtual_base: usize, virt_addr: usize, table_level: TableLevel) -> usize {
  match table_level {
    TableLevel::Level1 => {
      let mask = if virt_addr & virtual_base == virtual_base {
        if virtual_base == 0x8000_0000 {
          0x1usize
        } else {
          0x3usize
        }
      } else {
        0x3usize
      };

      ((virt_addr >> LEVEL_1_SHIFT_LONG) & mask) << 1
    },
    TableLevel::Level2 => ((virt_addr >> LEVEL_2_SHIFT_LONG) & INDEX_MASK_LONG) << 1,
    TableLevel::Level3 => ((virt_addr >> LEVEL_3_SHIFT_LONG) & INDEX_MASK_LONG) << 1,
  }
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
/// The table level must be 2 or 3. The Level 1 table can only point to Level 2
/// tables.
///
/// # Returns
///
/// A tuple with the low and high 32-bits of the descriptor. The high 32-bits
/// are zero if LPAE is not supported.
fn make_descriptor(
  table_level: TableLevel,
  phys_addr: usize,
  device: bool,
) -> Option<(usize, usize)> {
  let mair_idx = if device {
    MM_DEVICE_MAIR_IDX_LONG
  } else {
    MM_NORMAL_MAIR_IDX_LONG
  };

  match table_level {
    TableLevel::Level2 => Some(make_block_descriptor(phys_addr, mair_idx)),
    TableLevel::Level3 => Some(make_page_descriptor(phys_addr, mair_idx)),
    _ => None,
  }
}

/// Make a level 2 block descriptor.
///
/// # Parameters
///
/// * `phys_addr` - The physical address of the block or page.
/// * `mair_idx` - The block attributes MAIR index.
///
/// # Description
///
///   NOTE: A 2 MiB level 2 block descriptor requires LPAE.
///
/// # Returns
///
/// A tuple with the low and high 32-bits of the descriptor.
fn make_block_descriptor(phys_addr: usize, mair_idx: usize) -> (usize, usize) {
  (phys_addr | mair_idx | MM_ACCESS_FLAG_LONG | MM_BLOCK_FLAG_LONG, 0)
}

/// Make a level 2 or 3 page descriptor.
///
/// # Parameters
///
/// * `phys_addr` - The physical address of the block or page.
/// * `mair_idx` - The block attributes MAIR index.
///
/// # Description
///
///   NOTE: Assumes 4 KiB pages for both small and large page descriptors.
///
/// # Returns
///
/// A tuple with the low and high 32-bits of the descriptor. The high 32-bits
/// are zero if LPAE is not supported.
fn make_page_descriptor(phys_addr: usize, mair_idx: usize) -> (usize, usize) {
  (phys_addr | mair_idx | MM_ACCESS_FLAG_LONG | MM_PAGE_FLAG_LONG, 0)
}
