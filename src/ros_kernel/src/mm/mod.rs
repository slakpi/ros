//! Memory Management

mod page_allocator;
mod pager;

use crate::arch;

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
