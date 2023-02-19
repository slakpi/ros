use super::drivers::video::framebuffer;
use super::exceptions;
use super::mm::pages;
use super::peripherals::{base, mini_uart};
use crate::dbg_print;
use core::panic::PanicInfo;

/// @struct KernelConfig
/// @brief Basic kernel configuration provided by the bootstrap code. All
///        address are physical.
#[repr(C)]
pub struct KernelConfig {
  virtual_base: usize,
  page_size: usize,
  blob: usize,
  peripheral_base: usize,
  peripheral_block_size: usize,
  kernel_base: usize,
  kernel_size: usize,
  kernel_pages_start: usize,
  kernel_pages_size: usize,
}

/// @fn panic
/// @brief   Panic handler.
/// @param[in] info The panic info.
/// @returns Does not return.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
  dbg_print!("Kernel Panic: {}", info);
  loop {}
}

/// @fn ros_kernel
/// @brief   Kernel stub.
/// @param[in] config Kernel configuration struct.
/// @returns Does not return
#[no_mangle]
extern "C" fn ros_kernel(init: KernelConfig) -> ! {
  init_exceptions();
  init_peripherals(&init);

  dbg_print!("=== ROS ===\n");

  init_memory(&init);
  init_drivers();

  loop {}
}

fn init_exceptions() {
  exceptions::init_exception_vectors();
}

fn init_peripherals(init: &KernelConfig) {
  base::set_peripheral_base_addr(init.peripheral_base + init.virtual_base);
  mini_uart::init_uart();
}

fn init_memory(init: &KernelConfig) {
  pages::init_page_tables(
    init.virtual_base,
    init.blob,
    init.kernel_base,
    init.kernel_size,
    init.kernel_pages_start,
  );
}

/// @fn init_drivers
/// @brief Initialize drivers.
fn init_drivers() {
  framebuffer::fb_init();
}
