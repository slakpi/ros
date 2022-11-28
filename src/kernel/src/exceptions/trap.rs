/// @fn trap_exception(esr_el1: usize, far_el1: usize)
/// @brief Handles an exception trap.
/// @param[in] esr_el1 Exception Syndrome Register.
/// @param[in] far_el1 Fault Address Register.
/// cbindgen:ignore
#[no_mangle]
pub extern "C" fn trap_exception(_esr_el1: usize, _far_el1: usize) {}
