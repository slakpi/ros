//! ROS Kernel entry point.

#![no_std]

mod arch;
mod drivers;
mod mm;
mod peripherals;
mod support;
mod sync;
mod task;

#[cfg(feature = "module_tests")]
mod test;

use core::panic::PanicInfo;

/// Panic handler. Prints out diagnostic information and halts.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
  debug_print!("Kernel Panic: {}\n", info);
  loop {}
}

/// Kernel low-level initialization.
///
/// # Parameters
///
/// * `config` - Pointer to the kernel configuration struct provided by the
///   start code.
#[no_mangle]
extern "C" fn ros_kernel_init(config: usize) {
  // Initialize the architecture. At a minimum, this gives the kernel access to
  // all available memory and configures some method of debug output.
  arch::init(config);

  kernel_init();
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
