//! AArch64 Initialization

pub mod exceptions;
pub mod mm;
pub mod peripherals;

use crate::mm::page_allocator::PageAllocator;
use crate::peripherals::{memory, soc};

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

/// AArch64 platform configuration.
///
/// # Parameters
///
/// * `config` - The kernel configuration address provided by the bootstrap
///   code.
pub fn init(config: usize) {
  assert!(config != 0);

  let config = unsafe { &*(config as *const KernelConfig) };

  // TODO: 16 KiB and 64 KiB page support.
  assert!(config.page_size == 4096);

  let mem_layout = memory::get_memory_layout(config.virtual_base + config.blob).unwrap();
  let soc_layout = soc::get_soc_memory_layout(config.virtual_base + config.blob).unwrap();
  let mut pages_end = config.kernel_pages_start + config.kernel_pages_size;

  exceptions::init();

  pages_end = mm::init(
    config.virtual_base,
    config.kernel_pages_start,
    pages_end,
    &mem_layout,
  );

  pages_end = peripherals::init(
    config.virtual_base,
    config.kernel_pages_start,
    pages_end,
    &soc_layout,
  );

  for r in mem_layout.get_ranges() {
    _ = PageAllocator::new(
      config.page_size,
      r.base,
      r.size,
      (config.virtual_base + pages_end) as *mut u8,
    );
  }

  unsafe { MEM_LAYOUT = mem_layout };
}

pub fn _get_memory_layout() -> &'static memory::MemoryConfig {
  unsafe { &MEM_LAYOUT }
}
