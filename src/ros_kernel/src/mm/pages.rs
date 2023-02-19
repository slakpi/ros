#[cfg(target_arch = "aarch64")]
use crate::arch::aarch64 as arch;

#[cfg(target_arch = "arm")]
use crate::arch::armv7 as arch;

pub fn init_page_tables(
  virtual_base: usize,
  blob: usize,
  kernel_base: usize,
  kernel_size: usize,
  pages_start: usize,
) {
  arch::pages::init_page_tables(virtual_base, blob, kernel_base, kernel_size, pages_start);
}
