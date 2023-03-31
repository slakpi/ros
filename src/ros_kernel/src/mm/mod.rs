//! Memory Manager

#[cfg(target_arch = "aarch64")]
use crate::arch::aarch64 as arch;

#[cfg(target_arch = "arm")]
use crate::arch::armv7 as arch;

pub fn direct_map_memory(
  virtual_base: usize,
  pages_start: usize,
  pages_end: usize,
  base: usize,
  size: usize,
  device: bool,
) -> usize {
  arch::mm::direct_map_memory(virtual_base, pages_start, pages_end, base, size, device)
}
