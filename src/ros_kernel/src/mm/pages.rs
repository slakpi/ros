#[cfg(target_arch = "aarch64")]
use crate::arch::aarch64 as arch;

#[cfg(target_arch = "arm")]
use crate::arch::armv7 as arch;

static mut PAGES_START: usize = 0;
static mut PAGE_SIZE: usize = 0;

pub fn init_page_table(pages_start: usize, page_size: usize) {
  unsafe {
    debug_assert!(PAGES_START == 0);
    debug_assert!(PAGE_SIZE == 0);

    PAGES_START = pages_start;
    PAGE_SIZE = page_size;
  }

  arch::pages::init_page_table(pages_start, page_size);
}
