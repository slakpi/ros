use super::drivers::video::framebuffer;
use super::peripherals::{base, mailbox, mini_uart};
use super::support::dtb;
use crate::dbg_print;
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
/// @brief   Kernel stub.
/// @param[in] blob            ATAG or Device Tree blob.
/// @param[in] peripheral_base The peripheral base address.
/// @returns Does not return
#[no_mangle]
pub extern "C" fn kernel_stub(blob: u32, peripheral_base: u32) -> ! {
  base::set_peripheral_base_addr(peripheral_base as usize);
  mini_uart::uart_init();
  ros_kernel(blob as usize);
}

/// @fn ros_kernel(blob: usize) -> !
/// @brief   Rust kernel entry point.
/// @param[in] blob The ATAG or DTB blob pointer.
/// @returns Does not return.
fn ros_kernel(blob: usize) -> ! {
  dbg_print!("=== ROS ===\n");
  init_board();
  init_devices(blob);
  init_memory();
  init_peripherals();
  init_drivers();
  loop {}
}

fn init_board() {
  let (ok, model) = mailbox::get_board_model();

  if !ok {
    dbg_print!("Failed to get board model.\n");
    return;
  }

  let (ok, rev) = mailbox::get_board_revision();

  if !ok {
    dbg_print!("Failed to get board revision.\n");
    return;
  }

  dbg_print!("Raspberry Pi model {:#x}, rev {:#x}\n", model, rev);
}

fn init_devices(blob: usize) {
  let (valid_dtb, size) = dtb::check_dtb(blob as *const u8);

  if !valid_dtb {
    dbg_print!("Invalid dtb.\n");
  } else {
    dbg_print!("Found valid dtb at {:#x} with size {:#x}\n", blob as usize, size);
  }
}

/// @fn init_peripherals()
/// @brief Initialize peripheral devices.
fn init_peripherals() {
}

/// @fn init_memory()
fn init_memory() {
  let (ok, base, size) = mailbox::get_arm_memory();

  if !ok {
    dbg_print!("Failed to get low memory range.\n");
    return;
  }

  dbg_print!("Low memory: {:#x} {:#x}\n", base, size);
}

/// @fn init_drivers()
/// @brief Initialize drivers.
fn init_drivers() {
  framebuffer::fb_init();
}
