mod page_allocator;
mod paging;

use crate::arch;
use core::sync::atomic::{AtomicBool, Ordering};

/// Re-initialization guard for debug.
#[cfg(debug_assertions)]
static mut INITIALIZED: AtomicBool = AtomicBool::new(false);

pub fn init() {
  // initialization_guard();
  paging::init();
}

/// Verify that architecture initialization has not occurred.
///
/// # Description
///
///   NOTE: Debug only. The call should be compiled out with any optimization
///         enabled.
fn initialization_guard() {
  #[cfg(debug_assertions)]
  unsafe {
    debug_assert_eq!(
      INITIALIZED.compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed),
      Ok(false)
    );
  }
}
