use super::drivers::video::framebuffer;
use super::peripherals::{base, memory, mini_uart};
use core::panic::PanicInfo;

#[repr(C)]
pub struct KernelConfig {
  peripheral_base: usize,
  page_size: usize,
  kernel_base: usize,
  kernel_size: usize,
  kernel_pages_start: usize,
  kernel_pages_size: usize,
}

/// @fn panic(_info: &PanicInfo) -> !
/// @brief   Panic handler.
/// @param[in] info The panic info.
/// @returns Does not return.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
  loop {}
}

/// @fn ros_kernel(blob: usize, config: KernelConfig) -> ! {
/// @brief   Kernel stub.
/// @param[in] blob   ATAG or Device Tree blob.
/// @param[in] config Kernel configuration struct.
/// @returns Does not return
#[no_mangle]
pub extern "C" fn ros_kernel(blob: usize, config: KernelConfig) -> ! {
  base::set_peripheral_base_addr(config.peripheral_base);
  mini_uart::init_uart();
  memory::init_memory(
    blob as usize,
    config.page_size,
    config.kernel_base,
    config.kernel_size,
  );
  init_drivers();
  loop {}
}

/// @fn init_drivers()
/// @brief Initialize drivers.
fn init_drivers() {
  framebuffer::fb_init();
}
