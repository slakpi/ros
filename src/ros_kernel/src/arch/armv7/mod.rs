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

static mut MEM_LAYOUT: memory::MemoryConfig = memory::MemoryConfig::new();

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
}

pub fn _get_memory_layout() -> &'static memory::MemoryConfig {
  unsafe { &MEM_LAYOUT }
}
