//! ROS Kernel entry point.

use super::arch;
use super::mm;
use crate::debug_print;
use core::panic::PanicInfo;

/// Panic handler. Prints out diagnostic information and halts.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
  debug_print!("Kernel Panic: {}\n", info);
  loop {}
}

/// Kernel stub.
///
/// # Parameters
///
/// * `config` - Pointer to the kernel configuration struct provided by the
///   bootstrap
///
/// # Returns
///
/// Does not return.
#[no_mangle]
extern "C" fn ros_kernel(config: usize) -> ! {
  // Initialize the architecture. At a minimum, this gives the kernel access to
  // all available memory and configures some method of debug output.
  arch::init(config);

  kernel_init();

  loop {}
}

/// Kernel architecture-independent initialization.
#[cfg(not(feature = "module_tests"))]
fn kernel_init() {
  mm::init();
}

/// Dummy version to run module tests.
#[cfg(feature = "module_tests")]
fn kernel_init() {
  debug_print!("--- Running module tests  ---\n");
  mm::run_tests();
  debug_print!("--- Module tests complete ---\n");
}
