use crate::drivers::video::{console, framebuffer};
use crate::peripherals::{base, mini_uart};
use crate::support::atags;
use crate::support::kernel_init::ROSKernelInit;
use crate::{dbg_print, kprint};
use core::panic::PanicInfo;

/// @fn panic(_info: &PanicInfo) -> !
/// @brief   Panic handler.
/// @param[in] info The panic info.
/// @returns Does not return.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
  loop {}
}

/// @fn kernel_stub(blob: u32, peripheral_base: u32) -> !
/// @brief   AArch64 kernel stub.
/// @param[in] blob            ATAG or Device Tree blob.
/// @param[in] peripheral_base The peripheral base address.
/// @returns Does not return
#[cfg(target_arch = "aarch64")]
#[no_mangle]
pub extern "C" fn kernel_stub(_blob: u32, peripheral_base: u32) -> ! {
  let mut init = ROSKernelInit::new();
  init.peripheral_base = peripheral_base as usize;

  // TODO: Attempt to parse a device tree if this is not an ATAG list.
  // atags::read_atags(&mut init, blob);

  ros_kernel(init)
}

/// @fn kernel_stub(machine_id: u32, blob: u32, peripheral_base: u32) -> !
/// @brief   ARMv7 kernel stub.
/// @param[in] machine_id      The machine ID provided by the bootloader.
/// @param[in] blob            ATAG or Device Tree blob.
/// @param[in] peripheral_base The peripheral base address.
/// @returns Does not return
#[cfg(target_arch = "arm")]
#[no_mangle]
pub extern "C" fn kernel_stub(_machine_id: u32, _blob: u32, peripheral_base: u32) -> ! {
  let mut init = ROSKernelInit::new();
  init.peripheral_base = peripheral_base as usize;

  // TODO: Attempt to parse a device tree if this is not an ATAG list.
  // atags::read_atags(&mut init, blob);

  ros_kernel(init)
}

/// @fn ros_kernel(init: *const ROSKernelInit) -> !
/// @brief   Rust kernel entry point.
/// @param[in] init Architecture-dependent initialization parameters.
/// @returns Does not return.
fn ros_kernel(init: ROSKernelInit) -> ! {
  init_peripherals(&init);

  dbg_print!("=== ROS ===\n");
  dbg_print!("Peripheral Base Address: {:#x}\n", init.peripheral_base);

  init_drivers();

  console::clear();
  kprint!("=== ROS ===\n");
  kprint!("Peripheral Base Address: {:#x}\n", init.peripheral_base);

  loop {}
}

/// @fn init_peripherals(init: &ROSKernelInit)
/// @brief Initialize peripheral devices.
/// @param[in] init Architecture-dependent initialization parameters.
fn init_peripherals(init: &ROSKernelInit) {
  base::set_peripheral_base_addr(init.peripheral_base);
  mini_uart::uart_init();
}

/// @fn init_drivers()
/// @brief Initialize drivers.
fn init_drivers() {
  framebuffer::fb_init();
}
