use crate::dbg_print;
use core::arch::asm;

/// @fn init_execption_vectors
/// @brief AArch64 exception vector initializer.
pub fn init_exception_vectors() {
  unsafe {
    asm!("adr    x9, vectors", "msr    vbar_el1, x9",);
  }
}

/// @fn trap_exception
/// @brief AArch64 exception trap.
/// @param[in] esr_el1 Exception Syndrome Register.
/// @param[in] far_el1 Fault Address Register.
#[no_mangle]
extern "C" fn trap_exception(esr_el1: usize, far_el1: usize) {
  dbg_print!(
    "Fell into a trap! esr_el1={:#x}, far_el1={:#x}\n",
    esr_el1,
    far_el1,
  );
}
