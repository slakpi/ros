//! ARMv7a Initialization

pub mod bits;

mod exceptions;
mod mm;
mod peripherals;

use crate::peripherals::memory;
use core::sync::atomic::{AtomicBool, Ordering};

/// Basic kernel configuration provided by the bootstrap code. All address are
/// physical.
#[repr(C)]
struct KernelConfig {
  virtual_base: usize,
  page_size: usize,
  blob: usize,
  kernel_base: usize,
  kernel_size: usize,
  kernel_pages_start: usize,
  kernel_pages_size: usize,
}

/// Re-initialization guard for debug.
#[cfg(debug_assertions)]
static mut INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Layout of physical memory in the system.
static mut MEM_LAYOUT: memory::MemoryConfig = memory::MemoryConfig::new();

/// Layout of memory exclusions for the kernel area and DTB, if present.
static mut EXCL_LAYOUT: memory::MemoryConfig = memory::MemoryConfig::new();

/// Page size.
static mut PAGE_SIZE: usize = 0;

/// Kernel virtual address base.
static mut VIRTUAL_BASE: usize = 0;

/// ARMv7a platform configuration.
///
/// # Parameters
///
/// * `config` - The kernel configuration address provided by the bootstrap
///   code.
///
/// # Description
///
///   NOTE: Must only be called once.
///
/// Initializes the interrupt table, determines the physical memory layout,
/// initializes the kernel page tables, and builds a list of exclusions to the
/// physical memory layout.
pub fn init(config: usize) {
  initialization_guard();

  assert!(config != 0);

  let config = unsafe { &*(config as *const KernelConfig) };

  assert!(config.page_size == 4096);

  unsafe {
    PAGE_SIZE = config.page_size;
    VIRTUAL_BASE = config.virtual_base;
  }

  exceptions::init();
}

/// Get the physical memory layout.
///
/// # Description
///
///   NOTE: The interface guarantees read-only access outside of the module and
///         one-time initialization is assumed. Therefore, read access is safe
///         and sound.
pub fn get_memory_layout() -> &'static memory::MemoryConfig {
  unsafe { &MEM_LAYOUT }
}

/// Get the physical memory exclusion list.
///
/// # Description
///
///   NOTE: The interface guarantees read-only access outside of the module and
///         one-time initialization is assumed. Therefore, read access is safe
///         and sound.
pub fn get_exclusion_layout() -> &'static memory::MemoryConfig {
  unsafe { &EXCL_LAYOUT }
}

/// Get the page size.
///
/// # Description
///
///   NOTE: The interface guarantees read-only access outside of the module and
///         one-time initialization is assumed. Therefore, read access is safe
///         and sound.
pub fn get_page_size() -> usize {
  unsafe { PAGE_SIZE }
}

/// Get the kernel virtual base address.
///
/// # Description
///
///   NOTE: The interface guarantees read-only access outside of the module and
///         one-time initialization is assumed. Therefore, read access is safe
///         and sound.
pub fn get_kernel_virtual_base() -> usize {
  unsafe { VIRTUAL_BASE }
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
