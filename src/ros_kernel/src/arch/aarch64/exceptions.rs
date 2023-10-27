//! AArch64 Exception Management

use crate::debug_print;

/// AArch64 exception trap.
///
/// # Parameters
///
/// * `esr_el1` - Exception Syndrome Register value.
/// * `far_el1` - Fault Address Register value.
#[no_mangle]
extern "C" fn trap_exception(esr_el1: usize, far_el1: usize) {
  debug_print!(
    "Fell into a trap! esr_el1={:#x}, far_el1={:#x}\n",
    esr_el1,
    far_el1,
  );
}
