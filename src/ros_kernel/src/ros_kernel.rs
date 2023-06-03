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
/// * `config` - The kernel configuration address provided by the bootstrap
///   code.
///
/// # Returns
///
/// Does not return.
#[no_mangle]
extern "C" fn ros_kernel(config: usize) -> ! {
  arch::init(config);
  mm::init();

  #[cfg(feature = "unit_tests")]
  {
    mm::run_tests();
  }

  loop {}
}
