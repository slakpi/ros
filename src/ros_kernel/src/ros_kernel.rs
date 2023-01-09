use super::drivers::video::framebuffer;
use super::peripherals::{base, memory, mini_uart};
use super::support::{dtb, rpi};
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

/// @fn ros_kernel(blob: u32, peripheral_base: u32) -> !
/// @brief   Kernel stub.
/// @param[in] blob            ATAG or Device Tree blob.
/// @param[in] peripheral_base The peripheral base address.
/// @returns Does not return
#[no_mangle]
pub extern "C" fn ros_kernel(blob: u32, peripheral_base: u32, page_size: u32) -> ! {
  base::set_peripheral_base_addr(peripheral_base as usize);
  mini_uart::init_uart();

  dbg_print!("\n\n\n=== ROS ===\n");

  let rpi_confg = rpi::RpiConfig::new(peripheral_base as usize, blob as usize, page_size);

  memory::init_memory(&rpi_confg);

  init_drivers();

  loop {}
}

/// @fn init_drivers()
/// @brief Initialize drivers.
fn init_drivers() {
  framebuffer::fb_init();
}
