//! AArch64

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

/// AArch64 platform configuration.
///
/// # Parameters
///
/// * `config` - The kernel configuration address provided by the bootstrap
///   code.
pub fn init(config: usize) {
  debug_assert!(config != 0);

  let config = unsafe { &*(config as *const KernelConfig) };
  let mut pages_end = config.kernel_pages_start + config.kernel_pages_size;

  exceptions::init();

  pages_end = mm::init(
    config.virtual_base,
    config.blob,
    config.kernel_pages_start,
    pages_end,
  );

  _ = peripherals::init(
    config.virtual_base,
    config.blob,
    config.kernel_pages_start,
    pages_end,
  );
}
