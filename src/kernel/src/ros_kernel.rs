use crate::dbg_print;
use crate::peripherals;
use core::panic::PanicInfo;

/// @struct ROSKernelInit
/// @var ROSKernelInit::peripheral_base The base address for peripherals.
#[repr(C)]
pub struct ROSKernelInit {
  pub peripheral_base: usize,
}

/// @fn ros_kernel(init: *const ROSKernelInit) -> !
/// @brief   Rust kernel entry point.
/// @param[in] init Pointer to the architecture-dependent setup.
/// @returns Does not return.
#[no_mangle]
pub extern "C" fn ros_kernel(init: *const ROSKernelInit) -> ! {
  unsafe {
    assert!(!init.is_null());
    peripherals::base::set_peripheral_base_addr((*init).peripheral_base);
  }

  peripherals::mini_uart::uart_init();

  dbg_print!("=== ROS ===\n");
  dbg_print!(
    "Peripheral Base Address: {:#x}",
    peripherals::base::get_peripheral_register_addr(0) as usize
  );

  loop {}
}

/// @fn panic(_info: &PanicInfo) -> !
/// @brief   Panic handler.
/// @param[in] info The panic info.
/// @returns Does not return.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
  loop {}
}
