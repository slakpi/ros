pub mod exceptions;
pub mod mm;

use core::ffi::c_void;
use core::ptr;

/// Basic kernel configuration provided by the bootstrap code. All address are
/// physical.
#[repr(C)]
struct KernelConfig {
  virtual_base: usize,
  page_size: usize,
  blob: usize,
  peripheral_base: usize,
  peripheral_block_size: usize,
  kernel_base: usize,
  kernel_size: usize,
  kernel_pages_start: usize,
  kernel_pages_size: usize,
}

/// ARMv7a platform configuration.
///
/// # Parameters
///
/// `config` - The kernel configuration provided by the bootstrap code.
pub fn init(config: *const c_void) {
  debug_assert!(config != ptr::null());

  let config = &*(config as *const KernelConfig);

  exceptions::init();

  _ = mm::init(
    config.virtual_base,
    config.blob,
    config.kernel_pages_start,
    config.kernel_pages_end,
  );
}
