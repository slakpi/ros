//! ROS Kernel entry point.

use super::arch;
use super::peripherals::mini_uart;
use crate::dbg_print;
use core::ffi::c_void;
use core::panic::PanicInfo;

/// Panic handler. Prints out diagnostic information and halts.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
  dbg_print!("Kernel Panic: {}\n", info);
  loop {}
}

/// Kernel stub.
///
/// # Parameters
///
/// * `config` - The kernel configuration provided by the bootstrap code.
///
/// # Returns
///
/// Does not return.
#[no_mangle]
extern "C" fn ros_kernel(config: *const c_void) -> ! {
  arch::init(config);
  loop {}
}
