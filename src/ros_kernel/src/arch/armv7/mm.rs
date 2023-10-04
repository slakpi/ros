//! ARMv7a Memory Management

use super::task;
use crate::peripherals::memory;

/// Any physical memory below HIGH_MEMORY will be mapped directly into the
/// kernel's address space. The last 128 MiB of the kernel's address space will
/// act as a scratch area the kernel will use to map pages in the higher
/// physical memory regions.
///
/// Kernel segment split:
///
///   +-----------------+ 0xffff_ffff
///   |                 |
///   | High Memory     | Temporary mappings for [0x3800_0000, 0xc000_0000)
///   |                 |
///   +-----------------+ 0xf800_0000
///   +-----------------+ 0xf7ff_ffff maps directly to 0x37ff_ffff
///   |                 |
///   |                 |
///   | Low Memory      |
///   |                 |
///   |                 |
///   +-----------------+ 0xc000_0000 maps directly to 0x0000_0000
const HIGH_MEMORY: usize = 0x3800_0000;

/// Is high memory necessary (physical memory exceeds 896 MiB).
static mut USING_HIGH_MEMORY: bool = false;

/// Initialize kernel memory map.
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
/// space. Refer to `HIGH_MEMORY`. At most, only the first 896 MiB of physical
/// memory will be directly mapped into the kernel's virtual address space.
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
  pages_end
}

/// Initialize the ARMv7 page tables for the kernel. The canonical 32-bit 3:1
/// virtual address space layout for a process looks like:
///
///   +-----------------+ 0xffff_ffff
///   |                 |
///   | Kernel Segment  | 1 GiB
///   |                 |
///   +-----------------+ 0xc000_0000
///   |                 |
///   |                 |
///   |                 |
///   |                 |
///   | User Segment    | 3 GiB
///   |                 |
///   |                 |
///   |                 |
///   |                 |
///   +-----------------+ 0x0000_0000
///
/// This, of course, means the system is limited to accessing 3 GiB of physical
/// memory regardless of the actual amount of physical memory.
///
/// ARMv7a provides two independent registers for address translation so that
/// the kernel does not need to be mapped into the translation tables for every
/// process. The most-significant bit selects the register used for translation.
///
/// ARMv7a provides two levels of address space translation.
///
///     Level 1       ->  Level 2       
///     4096 Entries      256 Entries
///     Covers 4 GiB      Covers 1 MiB
///
/// ARMv7a allows using Level 1 entries to map 1 MiB sections with no Level 2
/// translation.
///
/// Each Level 2 table is 1 KiB in size and must be aligned to 1 KiB.
pub fn direct_map_memory(
  virtual_base: usize,
  pages_start: usize,
  pages_end: usize,
  base: usize,
  size: usize,
  device: bool,
) -> usize {
  pages_end
}

/// Map a range of physical addresses to a task's virtual address space.
///
/// # Parameters
///
/// * `virtual_base` - The task's virtual base address.
/// * `pages_start` - The address of the task's Level 1 page table.
/// * `pages_end` - The start of available memory for new pages.
/// * `virt` - Base of the virtual address range.
/// * `base` - Base of the physical address range.
/// * `size` - Size of the physical address range.
/// * `device` - Whether this block or page maps to device memory.
///
/// # Description
///
/// This is a generalized version of `direct_map_memory` where `virt` != `base`.
/// A physical address Ap maps the the virtual address
/// Av = virtual base + (Ap - base + virt).
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
  pages_end
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
/// TODO
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
/// * `virtual_base` - The kernel segment base address.
/// * `page` - The physical address of the page to map.
///
/// # Description
///
/// TODO
pub fn kernel_unmap_page_local(_: &mut task::Task, virtual_base: usize, page: usize) {
  debug_assert!(false);
}
