//! AArch64 exception handling.

use crate::dbg_print;
use core::arch::asm;

/// AArch64 exception vector initialization.
pub fn init() {
  unsafe {
    #[rustfmt::skip]
    asm!(
      "adr    x9, vectors",
      "msr    vbar_el1, x9",
    );
  }
}

/// AArch64 exception trap.
///
/// # Parameters
///
/// `esr_el1` - Exception Syndrome Register value.
/// `far_el1` - Fault Address Register value.
#[no_mangle]
extern "C" fn trap_exception(esr_el1: usize, far_el1: usize) {
  dbg_print!(
    "Fell into a trap! esr_el1={:#x}, far_el1={:#x}\n",
    esr_el1,
    far_el1,
  );
}
