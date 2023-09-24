//! Memory Management

mod page_allocator;
mod pager;

use crate::arch;
use crate::task;

#[cfg(feature = "module_tests")]
use crate::debug_print;

/// Re-initialization guard.
static mut INITIALIZED: bool = false;

/// Memory management initialization.
///
/// # Description
///
///   NOTE: Must only be called once while the kernel is single-threaded.
pub fn init() {
  unsafe {
    assert!(!INITIALIZED);
    INITIALIZED = true;
  }

  pager::init();
}

/// Maps a page into the kernel's virtual address space.
///
/// # Parameters
///
/// * `page` - The physical address of the page to map.
///
/// # Description
///
/// `kernel_map_page_local` must only be used to map pages into the kernel's
/// address space for the duration of a local procedure.
///
/// `kernel_unmap_page_local` must be used to unmap the page and pages must be
/// unmapped in the reverse order of calls to `kernel_map_page_local`.
///
/// # Returns
///
/// The virtual address of the mapped page.
pub fn kernel_map_page_local(page: usize) -> usize {
  let task = task::get_kernel_task();
  let virtual_base = arch::get_kernel_virtual_base();
  arch::mm::kernel_map_page_local(task, virtual_base, page)
}

/// Unmaps a page from the kernel's virtual address space.
///
/// # Parameters
///
/// * `page` - The physical address of the page to unmap.
///
/// # Description
///
/// `kernel_unmap_page_local` must only be used to unmap a page mapped using
/// `kernel_map_page_local`.
pub fn kernel_unmap_page_local(page: usize) {
  let task = task::get_kernel_task();
  let virtual_base = arch::get_kernel_virtual_base();
  arch::mm::kernel_unmap_page_local(task, virtual_base, page);
}

#[cfg(feature = "module_tests")]
pub fn run_tests() {
  debug_print!("[mm]\n");
  page_allocator::test::run_tests();
}
