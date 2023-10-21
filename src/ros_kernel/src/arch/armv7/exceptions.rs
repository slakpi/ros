//! ARMv7a Exception Handling

use crate::debug_print;

extern "C" {
  fn exp_move_exception_vectors(addr: usize);
}

/// ARMv7a exception vector initialization.
///
///   TODO: Moving the vectors is probably not necessary. The physical address
///         of the vectors can just be mapped to 0xffff_0000.
pub fn init() {
  unsafe { exp_move_exception_vectors(super::get_kernel_virtual_base()) };
}

/// ARMv7a exception trap.
///
///   TODO: Handle exceptions by type.
#[no_mangle]
extern "C" fn trap_exception() {
  debug_print!("Fell into a trap!");
}
