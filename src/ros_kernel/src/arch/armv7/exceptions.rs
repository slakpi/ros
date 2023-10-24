//! ARMv7a Exception Handling

use crate::debug_print;

/// ARMv7a exception trap.
///
///   TODO: Handle exceptions by type.
#[no_mangle]
extern "C" fn trap_exception() {
  debug_print!("Fell into a trap!");
}
