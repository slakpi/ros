//! ARMv7a Exception Handling

use crate::debug_print;

extern "C" {
  fn move_exception_vectors(addr: usize);
}

/// ARMv7a exception vector initialization.
///
/// # Description
///
/// Expects the exception vector table to be located at 0x0000_0000 which is
/// mapped to the kernel's virtual base address.
pub fn init() {
  unsafe { move_exception_vectors(super::get_kernel_virtual_base()) };
}

/// ARMv7a exception trap.
///
/// TODO: Handle exceptions by type.
#[no_mangle]
extern "C" fn trap_exception() {
  debug_print!("Fell into a trap!");
}
