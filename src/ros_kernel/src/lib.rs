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

/// Kernel initialization entry point.
///
/// # Parameters
///
/// * `config` - Pointer to the kernel configuration struct provided by the
///   start code.
///
/// # Description
///
/// This function should only be called by a single CPU to bootstrap the kernel
/// before enabling interrupts. All other CPUs should be gated.
///
/// After initializing the kernel, this function will ungate the remaining CPUs.
#[no_mangle]
extern "C" fn ros_kernel_init(config: usize) {
  // Initialize the architecture components. At a minimum, this gives the kernel
  // access to all available memory and configures some method of debug output.
  arch::init(config);

  // Initialize the architecture-independent components.
  kernel_init();

  // Bring secondary CPUs online.
  arch::init_secondary_cores();
}

/// Scheduler entry point.
///
/// # Description
///
/// Once the kernel has been bootstrapped, all CPUs should end up here to
/// request work. CPUs will also end up here in response to preemption timer
/// interrupts.
#[no_mangle]
extern "C" fn ros_kernel_scheduler() -> ! {
  // TODO: Anything other than this.
  debug_print!("Core waiting for work...\n");
  loop {}
}

/// Kernel architecture-independent component initialization.
fn kernel_init() {
  // Run module tests if configured.
  #[cfg(feature = "module_tests")]
  kernel_module_tests();

  // Initialize components.
  mm::init();
}

#[cfg(feature = "module_tests")]
fn kernel_module_tests() {
  debug_print!("--- Running module tests  ---\n");
  mm::run_tests();
  debug_print!("--- Module tests complete ---\n");
}
