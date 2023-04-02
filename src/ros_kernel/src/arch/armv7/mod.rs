//! ARMv7a

pub mod exceptions;
pub mod mm;
pub mod peripherals;

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

/// ARMv7a platform configuration.
///
/// # Parameters
///
/// * `config` - The kernel configuration address provided by the bootstrap
///   code.
pub fn init(config: usize) {
  debug_assert!(config != 0);

  let config = &*(config as *const KernelConfig);

  exceptions::init();

  _ = mm::init(
    config.virtual_base,
    config.blob,
    config.kernel_pages_start,
    config.kernel_pages_end,
  );
}
