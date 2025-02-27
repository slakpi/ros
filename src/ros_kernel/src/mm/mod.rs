//! Memory Management

mod buddy_allocator;
mod pager;
mod slab_allocator;

use crate::arch;
use crate::task;

#[cfg(feature = "module_tests")]
use crate::debug_print;

/// Re-initialization guard.
static mut INITIALIZED: bool = false;

/// Mapping strategies to use when mapping blocks of memory.
pub enum MappingStrategy {
  /// A strategy that uses architecture-specific techniques, such as ARM
  /// sections, to map a block of memory using the fewest table entries.
  Compact,
  /// A strategy that maps a block of memory to individual pages.
  Granular,
}

/// Interface for allocating table pages.
pub trait TableAllocator {
  /// Allocate a new table page.
  ///
  /// # Returns
  ///
  /// The physical address of the new table page.
  fn alloc_table(&mut self) -> usize;
}

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

/// Maps a page into the current thread's virtual address space.
///
/// # Parameters
///
/// * `page` - The physical address of the page to map.
///
/// # Description
///
/// Mappings must be kept local to a context and may only be unmapped in reverse
/// order.
///
/// # Returns
///
/// The virtual address of the mapped page.
pub fn kernel_map_page_local(page: usize) -> usize {
  let task = task::get_kernel_task();
  let virtual_base = arch::get_kernel_virtual_base();
  task.map_page_local(virtual_base, page)
}

/// Unmaps the current thread's last mapped page.
pub fn kernel_unmap_page_local() {
  let task = task::get_kernel_task();
  task.unmap_page_local();
}

pub fn kernel_allocate(pages: usize) -> Option<(usize, usize, usize)> {
  pager::allocate(pages)
}

pub fn kernel_free(base: usize, pages: usize, zone: usize) {
  pager::free(base, pages, zone);
}

#[cfg(feature = "module_tests")]
pub fn run_tests() {
  debug_print!("[mm]\n");
  page_allocator::test::run_tests();
}
