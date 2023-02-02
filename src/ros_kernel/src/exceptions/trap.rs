use crate::dbg_print;

/// @fn trap_exception
/// @brief Handles an exception trap.
/// @param[in] esr_el1 Exception Syndrome Register.
/// @param[in] far_el1 Fault Address Register.
#[no_mangle]
pub extern "C" fn trap_exception(esr_el1: usize, far_el1: usize) {
  dbg_print!(
    "Fell into a trap! esr_el1={:#x}, far_el1={:#x}\n",
    esr_el1,
    far_el1
  );
}
