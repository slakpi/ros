//! ARMv7a Memory Management
//!
//!   NOTE: Assumes that the kernel is running on a CPU that supports Large
//!         Physical Address Extensions and long page table descriptors.

use super::task;
use core::ptr;

const TABLE_SIZE: usize = 4096;
const PAGE_SHIFT: usize = 12;
const PAGE_SIZE: usize = 1 << PAGE_SHIFT;
const PAGE_MASK: usize = PAGE_SIZE - 1;
const SECTION_SHIFT: usize = 21;
const SECTION_SIZE: usize = 1 << SECTION_SHIFT;
const SECTION_MASK: usize = SECTION_SIZE - 1;
const BLOCK_SHIFT: usize = 30;
const BLOCK_SIZE: usize = 1 << BLOCK_SHIFT;

const TABLE_ADDR_MASK: usize = 0xffff_fc00;
const TYPE_MASK: usize = 0x3;
const MM_PAGE_TABLE_FLAG: usize = 0x1 << 0;
const MM_BLOCK_FLAG: usize = 0x1 << 1;

/// Physical start address of the high memory area.
const HIGH_MEMORY: usize = 0x3800_0000;

/// Translation table level.
#[derive(Clone, Copy, PartialEq)]
enum TableLevel {
  Level1,
  Level2,
  Level3,
}

/// Page table structure for 4 KiB pages. The Level 1 table only has four real
/// entries, but occupies an entire page for alignment. Note that the array uses
/// 64-bit entries explicitly rather than pointer-sized entries. Long page table
/// descriptors are 64-bit even with the pointer size is 32-bit.
#[repr(C)]
struct PageTable {
  entries: [u64; 512],
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
  )
}

/// Map a range of physical addresses to a task's virtual address space.
///
/// # Parameters
///
/// * `virtual_base` - The task's virtual base address.
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
    TableLevel::Level1 => BLOCK_SIZE,
    TableLevel::Level2 => SECTION_SIZE,
    TableLevel::Level3 => PAGE_SIZE,
  }
}

/// Get the physical address for either the next table from a descriptor.
///
/// # Parameters
///
/// * `desc` - The descriptor.
///
/// # Returns
///
/// The physical address.
fn get_phys_addr_from_descriptor(desc: usize) -> usize {
  desc & TABLE_ADDR_MASK
}

/// Allocates a new page table if necessary, then fills the table with entries
/// for the specified range of memory.
///
/// # Parameters
///
/// * `virtual_base` - The kernel segment base address.
/// * `desc` - The current descriptor in the table.
/// * `pages_end` - The current end of the table area.
/// * `virt` - Base of the virtual address range.
/// * `base` - Base of the physical address range.
/// * `size` - Size of the physical address range.
/// * `device` - Whether this block or page maps to device memory.
///
/// # Description
///
///     NOTE: This function assumes that the descriptor is an entry in the
///           Level 1 page table.
///
/// # Returns
///
/// The new end of the table area.
fn alloc_table_and_fill(
  virtual_base: usize,
  desc: usize,
  pages_end: usize,
  virt: usize,
  base: usize,
  size: usize,
  device: bool,
) -> (usize, usize) {
  let mut next_addr = get_phys_addr_from_descriptor(desc);
  let mut desc = desc;
  let mut pages_end = pages_end;

  // TODO: It is probably fine to overwrite a section descriptor. If the memory
  //       configuration is overwriting itself, then we probably have something
  //       wrong and a memory trap is the right outcome.
  if desc & TYPE_MASK != MM_PAGE_TABLE_FLAG {
    next_addr = pages_end;
    pages_end += TABLE_SIZE;

    unsafe {
      // Zero out the table. Any entry in the table with bits 0 and 1 set to 0
      // is invalid.
      ptr::write_bytes((virtual_base + next_addr) as *mut u8, 0, TABLE_SIZE);
    }

    // desc = make_pointer_entry(next_addr);
  }

  (
    desc,
    fill_table(
      virtual_base,
      TableLevel::Level2,
      next_addr,
      pages_end,
      virt,
      base,
      size,
      device,
    ),
  )
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
  pages_end
}
