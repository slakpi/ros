//! Memory Manager

#[cfg(target_arch = "aarch64")]
use crate::arch::aarch64 as arch;

#[cfg(target_arch = "arm")]
use crate::arch::armv7 as arch;

use crate::peripherals::memory;

pub fn init_memory(virtual_base: usize, pages_start: usize, mem_config: &memory::MemoryConfig) {
  arch::mm::init_memory(virtual_base, pages_start, mem_config);
}
