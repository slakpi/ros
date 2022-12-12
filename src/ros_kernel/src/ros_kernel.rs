use crate::dbg_print;
use crate::drivers::video;
use crate::peripherals;
use crate::support::atags;
use crate::support::kernel_init::ROSKernelInit;
use core::panic::PanicInfo;

/// @fn panic(_info: &PanicInfo) -> !
/// @brief   Panic handler.
/// @param[in] info The panic info.
/// @returns Does not return.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
  loop {}
}

/// @fn kernel_stub(blob: usize, peripheral_base: usize) -> !
/// @brief   AArch64 kernel stub.
/// @param[in] blob            ATAG or Device Tree blob.
/// @param[in] peripheral_base The peripheral base address.
/// @returns Does not return
#[cfg(target_arch = "aarch64")]
#[no_mangle]
pub extern "C" fn kernel_stub(blob: usize, peripheral_base: usize) -> ! {
  let mut init = ROSKernelInit::new();
  init.peripheral_base = peripheral_base;

  // TODO: Attempt to parse a device tree if this is not an ATAG list.
  atags::read_atags(&mut init, blob);

  ros_kernel(init)
}

/// @fn kernel_stub(zero: usize, machine_id: usize, blob: usize, peripheral_base: usize) -> !
/// @brief   ARMv7 kernel stub.
/// @param[in] machine_id      The machine ID provided by the bootloader.
/// @param[in] blob            ATAG or Device Tree blob.
/// @param[in] peripheral_base The peripheral base address.
/// @returns Does not return
#[cfg(target_arch = "arm")]
#[no_mangle]
pub extern "C" fn kernel_stub(_machine_id: usize, blob: usize, peripheral_base: usize) -> ! {
  let mut init = ROSKernelInit::new();
  init.peripheral_base = peripheral_base;

  // TODO: Attempt to parse a device tree if this is not an ATAG list.
  atags::read_atags(&mut init, blob);

  ros_kernel(init)
}

/// @fn ros_kernel(init: *const ROSKernelInit) -> !
/// @brief   Rust kernel entry point.
/// @param[in] init Pointer to the architecture-dependent setup.
/// @returns Does not return.
fn ros_kernel(init: ROSKernelInit) -> ! {
  peripherals::base::set_peripheral_base_addr(init.peripheral_base);
  peripherals::mini_uart::uart_init();
  video::framebuffer::fb_init();
  startup_messages();
  loop {}
}

/// @fn startup_messages()
/// @brief Print some boring information on startup.
fn startup_messages() {
  let pbase = peripherals::base::get_peripheral_register_addr(0) as usize;
  dbg_print!("=== ROS ===\n");
  dbg_print!("Peripheral Base Address: {:#x}\n", pbase);

  video::framebuffer::draw_string("Hello, World!", 0, 0, 0x0f);
}
