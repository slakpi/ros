//! ARMv7a Architecture

pub mod debug;
pub mod exceptions;
pub mod mm;
pub mod peripherals;
pub mod task;

use crate::peripherals::memory;
use crate::support::bits;

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

/// Re-initialization guard.
static mut INITIALIZED: bool = false;

/// Layout of physical memory in the system.
static mut MEM_LAYOUT: memory::MemoryConfig = memory::MemoryConfig::new();

/// Layout of memory exclusions for the kernel area and DTB, if present.
static mut EXCL_LAYOUT: memory::MemoryConfig = memory::MemoryConfig::new();

/// Page size.
static mut PAGE_SIZE: usize = 0;

/// Page shift.
static mut PAGE_SHIFT: usize = 0;

/// Kernel virtual address base.
static mut VIRTUAL_BASE: usize = 0;

/// Max physical address.
static mut MAX_PHYSICAL_ADDRESS: usize = 0;

/// ARMv7a platform configuration.
///
/// # Parameters
///
/// * `config` - The kernel configuration address provided by the bootstrap
///   code.
///
/// # Description
///
///   NOTE: Must only be called once while the kernel is single-threaded.
///
/// Initializes the interrupt table, determines the physical memory layout,
/// initializes the kernel page tables, and builds a list of exclusions to the
/// physical memory layout.
pub fn init(config: usize) {
  unsafe {
    assert!(!INITIALIZED);
    INITIALIZED = true;
  }

  assert!(config != 0);

  let config = unsafe { &*(config as *const KernelConfig) };

  assert!(config.page_size == 4096);

  unsafe {
    PAGE_SIZE = config.page_size;
    PAGE_SHIFT = bits::floor_log2(config.page_size);
    VIRTUAL_BASE = config.virtual_base;
    MAX_PHYSICAL_ADDRESS = !VIRTUAL_BASE;
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

/// Get the page shift.
///
/// # Description
///
///   NOTE: The interface guarantees read-only access outside of the module and
///         one-time initialization is assumed. Therefore, read access is safe
///         and sound.
pub fn get_page_shift() -> usize {
  unsafe { PAGE_SHIFT }
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

/// Get the maximum physical address allowed.
///
/// # Description
///
///   NOTE: The interface guarantees read-only access outside of the module and
///         one-time initialization is assumed. Therefore, read access is safe
///         and sound.
///
/// # Returns
///
/// Returns the bitwise NOT of the kernel base address.
pub fn get_max_physical_address() -> usize {
  unsafe { MAX_PHYSICAL_ADDRESS }
}
