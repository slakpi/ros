//! ARMv7a Memory Management

use super::task;
use core::{cmp, ptr, slice};

extern "C" {
  fn ext_has_long_descriptor_support() -> usize;
}

const PAGE_SHIFT: usize = 12;
const PAGE_SIZE: usize = 1 << PAGE_SHIFT;
const PAGE_MASK: usize = PAGE_SIZE - 1;

const LEVEL_1_SHIFT_SHORT: usize = 20;
const LEVEL_1_INDEX_SHIFT_SHORT: usize = 12;
const LEVEL_1_INDEX_MASK_SHORT: usize = (1 << LEVEL_1_INDEX_SHIFT_SHORT) - 1;
const LEVEL_2_SHIFT_SHORT: usize = 12;
const LEVEL_2_INDEX_SHIFT_SHORT: usize = 8;
const LEVEL_2_INDEX_MASK_SHORT: usize = (1 << LEVEL_2_INDEX_SHIFT_SHORT) - 1;

const LEVEL_1_SHIFT_LONG: usize = 30;
const LEVEL_2_SHIFT_LONG: usize = 21;
const LEVEL_3_SHIFT_LONG: usize = 12;
const INDEX_SHIFT_LONG: usize = 9;
const INDEX_MASK_LONG: usize = (1 << INDEX_SHIFT_LONG) - 1;

/// Without LPAE, the Level 1 table has 4096 32-bit entries and the Level 2
/// table has 256 32-bit entries.
const LEVEL_1_TABLE_SIZE_SHORT: usize = 4096 * 4;
const LEVEL_2_TABLE_SIZE_SHORT: usize = 256 * 4;

/// With LPAE, the Level 1 table only has 4 entries, but let it use an entire
/// 4 KiB page so that it matches the Level 2 and 3 table sizes.
const TABLE_SIZE_LONG: usize = 512 * 8;

const ADDR_MASK_SHORT: usize = 0xffff_fc00;
const MM_PAGE_TABLE_FLAG_SHORT: usize = 0x1 << 0;
const MM_BLOCK_FLAG_SHORT: usize = 0x2 << 0;
const MM_PAGE_FLAG_SHORT: usize = 0x2 << 0;
const MM_L1_ACCESS_FLAG_SHORT: usize = 0x1 << 10;
const MM_L1_ACCESS_RW_SHORT: usize = 0x0 << 15;
const MM_L1_ACCESS_RO_SHORT: usize = 0x1 << 15;
const MM_L2_ACCESS_FLAG_SHORT: usize = 0x1 << 4;
const MM_L2_ACCESS_RW_SHORT: usize = 0x0 << 9;
const MM_L2_ACCESS_RO_SHORT: usize = 0x1 << 9;
const MM_DEVICE_MEM_SHORT: usize = 0x1 << 2;
const MM_NORMAL_MEM_SHORT: usize = 0x2 << 2;

const ADDR_MASK_LONG: usize = 0xffff_f000;
const MM_PAGE_TABLE_FLAG_LONG: usize = 0x3 << 0;
const MM_BLOCK_FLAG_LONG: usize = 0x1 << 0;
const MM_PAGE_FLAG_LONG: usize = 0x3 << 0;

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
  fill_table(
    virtual_base,
    TableLevel::Level1,
    pages_start,
    pages_end,
    base,
    base,
    size,
    device,
    has_lpae(),
  )
}

/// Map a range of physical addresses to a task's virtual address space.
///
/// # Parameters
///
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
    TableLevel::Level1,
    pages_start,
    pages_end,
    virt,
    base,
    size,
    device,
    has_lpae(),
  )
}

/// Maps a page into the kernel's virtual address space.
///
/// # Parameters
///
/// * `task` - The kernel task receiving the mapping.
/// * `virtual_base` - The kernel segment base address.
/// * `page` - The physical address of the page to map.
///
/// # Description
///
/// If the page is in low memory, the function simply returns the virtual
/// address of the mapped page without modifying the kernel's page table.
///
/// Otherwise, the function maps the page to the next available virtual address
/// in the task's local mappings. The mappings are thread-local, so the function
/// is thread safe.
///
///   NOTE: The Linux implementation ensures the thread is pinned to the same
///         CPU for the duration of temporary mappings.
///
/// The function will panic if no more pages can be mapped into the thread's
/// local mappings.
///
/// # Returns
///
/// The virtual address of the mapped page.
pub fn kernel_map_page_local(_: &mut task::Task, virtual_base: usize, page: usize) -> usize {
  debug_assert!(false);
  0
}

/// Unmaps a page from the kernel's virtual address space.
///
/// # Parameters
///
/// * `task` - The kernel task receiving the mapping.
///
/// # Description
///
/// If the page is in low memory or if no pages have been mapped into the
/// thread's local mappings, the function simply returns without modifying
/// the kernel's page table.
///
/// Otherwise, the function unmaps the page from the task's local mappings. The
/// mappings are thread-local, so the function is thread safe.
pub fn kernel_unmap_page_local(_: &mut task::Task) {
  debug_assert!(false);
}

/// Check if the CPU supports Large Physical Address Extensions.
///
/// # Returns
///
/// True if the CPU supports LPAE, false otherwise.
fn has_lpae() -> bool {
  unsafe { ext_has_long_descriptor_support() == 0 }
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
/// * `use_lpae` - Use Large Physical Address Extensions.
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
  use_lpae: bool,
) -> (usize, usize, usize) {
  let next_level = get_next_table(table_level, use_lpae).unwrap();
  let mut next_addr = get_phys_addr_from_descriptor(desc, desc_high, use_lpae);
  let mut desc = desc;
  let mut desc_high = desc_high;
  let mut pages_end = pages_end;

  // TODO: It is probably fine to overwrite a section descriptor. If the memory
  //       configuration is overwriting itself, then we probably have something
  //       wrong and a memory trap is the right outcome.
  if is_pointer_entry(desc, desc_high, use_lpae) {
    let table_size = get_table_size(table_level, use_lpae).unwrap();
    next_addr = pages_end;
    pages_end += table_size;

    unsafe {
      // Zero out the table. Any entry in the table with bits 0 and 1 set to 0
      // is invalid.
      ptr::write_bytes((virtual_base + next_addr) as *mut u8, 0, table_size);
    }

    (desc, desc_high) = make_pointer_entry(next_addr, use_lpae);
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
      use_lpae,
    ),
  )
}

/// Given a table level, return the next table level down in the translation
/// hierarchy.
///
/// # Parameters
///
/// * `table_level` - The current table level.
/// * `use_lpae` - Use Large Physical Address Extensions.
///
/// # Returns
///
/// The next table level. None if Level 2 is specified (no LPAE) or Level 3 is
/// is specified (with LPAE).
fn get_next_table(table_level: TableLevel, use_lpae: bool) -> Option<TableLevel> {
  match table_level {
    TableLevel::Level1 => Some(TableLevel::Level2),

    TableLevel::Level2 => {
      if use_lpae {
        Some(TableLevel::Level3)
      } else {
        None
      }
    }

    TableLevel::Level3 => None,
  }
}

/// Get the physical address for either the next table from a descriptor.
///
/// # Parameters
///
/// * `desc` - The descriptor.
/// * `desc_high` - High 32-bits of a long descriptor (0 if LPAE not supported).
/// * `use_lpae` - Use Large Physical Address Extensions.
///
/// # Description
///
///   NOTE: Does not support LPAE 40-bit pointers. The high 32-bits of the
///         descriptor are ignored.
///
/// # Returns
///
/// The physical address.
fn get_phys_addr_from_descriptor(desc: usize, _desc_high: usize, use_lpae: bool) -> usize {
  if use_lpae {
    desc & ADDR_MASK_LONG
  } else {
    desc & ADDR_MASK_SHORT
  }
}

/// Given a table level, determine the size of the table.
///
/// # Parameters
///
/// * `table_level` - The current table level.
/// * `use_lpae` - Use Large Physical Address Extensions.
///
/// # Returns
///
/// The size of the table in bytes.
fn get_table_size(table_level: TableLevel, use_lpae: bool) -> Option<usize> {
  match table_level {
    TableLevel::Level1 => {
      if use_lpae {
        Some(TABLE_SIZE_LONG)
      } else {
        Some(LEVEL_1_TABLE_SIZE_SHORT)
      }
    }

    TableLevel::Level2 => {
      if use_lpae {
        Some(TABLE_SIZE_LONG)
      } else {
        Some(LEVEL_2_TABLE_SIZE_SHORT)
      }
    }

    TableLevel::Level3 => {
      if use_lpae {
        Some(TABLE_SIZE_LONG)
      } else {
        None
      }
    }
  }
}

/// Determine if a descriptor is a table pointer entry.
///
/// # Parameters
///
/// * `desc` - The current descriptor in the table.
/// * `desc_high` - High 32-bits of a long descriptor (0 if LPAE not supported).
/// * `use_lpae` - Use Large Physical Address Extensions.
///
/// # Returns
///
/// True if the descriptor is a page table pointer, false otherwise.
fn is_pointer_entry(desc: usize, _desc_high: usize, use_lpae: bool) -> bool {
  if use_lpae {
    desc & TYPE_MASK == MM_PAGE_TABLE_FLAG_LONG
  } else {
    desc & TYPE_MASK == MM_PAGE_TABLE_FLAG_SHORT
  }
}

/// Make a pointer entry to a lower level page table.
///
/// # Parameters
///
/// * `phys_addr` - The physical address of the table.
/// * `use_lpae` - Use Large Physical Address Extensions.
///
/// # Returns
///
/// A tuple with the low and high 32-bits of the descriptor. The high 32-bits
/// are zero if LPAE is not supported.
fn make_pointer_entry(phys_addr: usize, use_lpae: bool) -> (usize, usize) {
  if use_lpae {
    ((phys_addr & ADDR_MASK_LONG) | MM_PAGE_TABLE_FLAG_LONG, 0)
  } else {
    ((phys_addr & ADDR_MASK_SHORT) | MM_PAGE_TABLE_FLAG_SHORT, 0)
  }
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
/// * `use_lpae` - Use Large Physical Address Extensions.
///
/// # Details
///
///     TODO: For now, memory management will just assume 4 KiB pages. The
///           bootstrap code will have already configured the MMU and provided
///           the page size in the kernel configuration struct.
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
/// 2 GiB (even 2:2 split).
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
/// user address space making a 3:1 split possible.
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
  use_lpae: bool,
) -> usize {
  let entry_size = get_table_entry_size(table_level, use_lpae).unwrap();
  let mut virt = virt;
  let mut base = base;
  let mut size = size;
  let mut pages_end = pages_end;

  while size >= PAGE_SIZE {
    let idx = get_descriptor_index(virtual_base + virt, table_level, use_lpae).unwrap();
    let table = get_table(table_level, table_addr, use_lpae).unwrap();
    let mut fill_size = entry_size;
    let desc: usize;
    let desc_high: usize;

    if size < entry_size || table_level == TableLevel::Level1 {
      fill_size = cmp::min(size, entry_size);

      if use_lpae {
        (desc, desc_high, pages_end) = alloc_table_and_fill(
          virtual_base,
          table_level,
          table[idx] as usize,
          table[idx + 1] as usize,
          pages_end,
          virt,
          base,
          fill_size,
          device,
          use_lpae,
        );
      } else {
        (desc, desc_high, pages_end) = alloc_table_and_fill(
          virtual_base,
          table_level,
          table[idx] as usize,
          0,
          pages_end,
          virt,
          base,
          fill_size,
          device,
          use_lpae,
        );
      }
    } else {
      (desc, desc_high) = make_descriptor(table_level, base, device, use_lpae).unwrap();
    }

    table[idx] = desc as u32;

    if use_lpae {
      table[idx + 1] = desc_high as u32;
    }

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
/// * `use_lpae` - Use Large Physical Address Extensions.
///
/// # Returns
///
/// The size covered by a single entry in bytes. None if the CPU does not have
/// LPAE and Level 3 is specified.
fn get_table_entry_size(table_level: TableLevel, use_lpae: bool) -> Option<usize> {
  match table_level {
    TableLevel::Level1 => {
      if use_lpae {
        Some(1 << LEVEL_1_SHIFT_LONG)
      } else {
        Some(1 << LEVEL_1_SHIFT_SHORT)
      }
    }

    TableLevel::Level2 => {
      if use_lpae {
        Some(1 << LEVEL_2_SHIFT_LONG)
      } else {
        Some(1 << LEVEL_2_SHIFT_SHORT)
      }
    }

    TableLevel::Level3 => {
      if use_lpae {
        Some(PAGE_SIZE)
      } else {
        None
      }
    }
  }
}

/// Get a memory slice for the table at a given address.
///
/// # Parameters
///
/// * `table_level` - The table level of interest.
/// * `table_addr` - The table address.
/// * `use_lpae` - Use Large Physical Address Extensions.
///
/// # Returns
///
/// A slice of the correct size for the table level, or None if the table level
/// is not valid.
fn get_table(
  table_level: TableLevel,
  table_addr: usize,
  use_lpae: bool,
) -> Option<&'static mut [u32]> {
  if use_lpae {
    // Note the shift right by 2 instead of 3. The slice is u32, not u64.
    return unsafe {
      Some(slice::from_raw_parts_mut(
        table_addr as *mut u32,
        TABLE_SIZE_LONG >> 2,
      ))
    };
  }

  match table_level {
    TableLevel::Level1 => unsafe {
      Some(slice::from_raw_parts_mut(
        table_addr as *mut u32,
        LEVEL_1_TABLE_SIZE_SHORT >> 2,
      ))
    },

    TableLevel::Level2 => unsafe {
      Some(slice::from_raw_parts_mut(
        table_addr as *mut u32,
        LEVEL_2_TABLE_SIZE_SHORT >> 2,
      ))
    },

    _ => None,
  }
}

/// Get the descriptor index for a virtual address in the specified table.
///
/// # Parameters
///
/// * `virt_addr` - The virtual address.
/// * `table_level` - The table level for the index.
/// * `use_lpae` - Use Large Physical Address Extensions.
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
fn get_descriptor_index(
  virt_addr: usize,
  table_level: TableLevel,
  use_lpae: bool,
) -> Option<usize> {
  match table_level {
    TableLevel::Level1 => {
      if use_lpae {
        Some(((virt_addr >> LEVEL_1_SHIFT_LONG) & INDEX_MASK_LONG) << 1)
      } else {
        Some((virt_addr >> LEVEL_1_SHIFT_SHORT) & LEVEL_1_INDEX_MASK_SHORT)
      }
    }

    TableLevel::Level2 => {
      if use_lpae {
        Some(((virt_addr >> LEVEL_2_SHIFT_LONG) & INDEX_MASK_LONG) << 1)
      } else {
        Some((virt_addr >> LEVEL_2_SHIFT_SHORT) & LEVEL_2_INDEX_MASK_SHORT)
      }
    }

    TableLevel::Level3 => {
      if use_lpae {
        Some(((virt_addr >> LEVEL_3_SHIFT_LONG) & INDEX_MASK_LONG) << 1)
      } else {
        None
      }
    }
  }
}

/// Create a table descriptor appropriate to the specified table level.
///
/// # Parameters
///
/// * `table_level` - The table level of the new entry.
/// * `phys_addr` - The physical address of the block or page.
/// * `device` - Whether this block or page maps to device memory.
/// * `use_lpae` - Use Large Physical Address Extensions.
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
  use_lpae: bool,
) -> Option<(usize, usize)> {
  match table_level {
    TableLevel::Level2 => {
      if use_lpae {
        Some(make_block_descriptor(phys_addr, device))
      } else {
        Some(make_page_descriptor(phys_addr, device, use_lpae))
      }
    }

    TableLevel::Level3 => {
      if use_lpae {
        Some(make_page_descriptor(phys_addr, device, use_lpae))
      } else {
        None
      }
    }

    _ => None,
  }
}

/// Make a level 2 block descriptor.
///
/// # Parameters
///
/// * `phys_addr` - The physical address of the block or page.
/// * `device` - Whether this block or page maps to device memory.
///
/// # Description
///
///   NOTE: A 2 MiB level 2 block descriptor requires LPAE.
///
/// # Returns
///
/// A tuple with the low and high 32-bits of the descriptor.
fn make_block_descriptor(phys_addr: usize, device: bool) -> (usize, usize) {
  (0, 0)
}

/// Make a level 2 or 3 page descriptor.
///
/// # Parameters
///
/// * `phys_addr` - The physical address of the block or page.
/// * `device` - Whether this block or page maps to device memory.
/// * `use_lpae` - Use Large Physical Address Extensions.
///
/// # Description
///
///   NOTE: Assumes 4 KiB pages for both small and large page descriptors.
///
/// # Returns
///
/// A tuple with the low and high 32-bits of the descriptor. The high 32-bits
/// are zero if LPAE is not supported.
fn make_page_descriptor(phys_addr: usize, device: bool, use_lpae: bool) -> (usize, usize) {
  if use_lpae {
    return (0, 0);
  } else {
    let mut entry = (phys_addr & ADDR_MASK_SHORT) | MM_L2_ACCESS_FLAG_SHORT | MM_NORMAL_MEM_SHORT;

    if device {
      entry |= MM_DEVICE_MEM_SHORT;
    }
  
    return (entry, 0)
  }
}
