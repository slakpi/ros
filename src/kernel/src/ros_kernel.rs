use crate::dbg_print;
use crate::peripherals;
use core::panic::PanicInfo;

/// @struct ROSMemoryRegion
/// @brief Memory region available to the kernel.
/// @var base The base address of the region.
/// @var size The size of the region in bytes.
#[repr(C)]
pub struct ROSMemoryRegion {
  pub base: usize,
  pub size: usize,
}

/// @struct ROSKernelInit
/// @brief Initialization parameters provided by the bootloader.
/// @var peripheral_base The base address for peripherals.
#[repr(C)]
pub struct ROSKernelInit {
  pub peripheral_base: usize,
  pub memory_regions: [ROSMemoryRegion; 16],
}

/// @fn panic(_info: &PanicInfo) -> !
/// @brief   Panic handler.
/// @param[in] info The panic info.
/// @returns Does not return.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
  loop {}
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
  startup_messages();
  loop {}
}

/// @fn trap_exception(esr_el1: usize, far_el1: usize)
/// @brief Handles an exception trap.
/// @param[in] esr_el1 Exception Syndrome Register.
/// @param[in] far_el1 Fault Address Register.
#[no_mangle]
pub extern "C" fn trap_exception(esr_el1: usize, far_el1: usize) {

}

fn startup_messages() {
  let pbase = peripherals::base::get_peripheral_register_addr(0) as usize;
  dbg_print!("=== ROS ===\n");
  dbg_print!("Peripheral Base Address: {:#x}\n", pbase);
}
