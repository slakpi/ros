//! Memory Management

mod page_allocator;
mod pager;

use crate::arch;
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

#[cfg(feature = "module_tests")]
pub fn run_tests() {
  debug_print!("[mm]\n");
  page_allocator::test::run_tests();
}
