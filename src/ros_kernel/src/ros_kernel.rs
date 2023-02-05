use super::drivers::video::framebuffer;
use super::exceptions;
use super::peripherals::{base, memory, mini_uart};
use crate::dbg_print;
use core::panic::PanicInfo;

/// @struct KernelConfig
/// @brief Basic kernel configuration provided by the bootstrap code. All
///        address are physical. The initial virtual addressing scheme setup by
///        the bootstrap code uses the physical addresses as offsets from the
///        virtual base.
#[repr(C)]
pub struct KernelConfig {
  virtual_base: usize,
  page_size: usize,
  blob: usize,
  peripheral_base: usize,
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
fn panic(_info: &PanicInfo) -> ! {
  loop {}
}

/// @fn ros_kernel
/// @brief   Kernel stub.
/// @param[in] config Kernel configuration struct.
/// @returns Does not return
#[no_mangle]
extern "C" fn ros_kernel(config: KernelConfig) -> ! {
  exceptions::init_exception_vectors();

  base::set_peripheral_base_addr(config.peripheral_base + config.virtual_base);
  mini_uart::init_uart();

  dbg_print!("=== ROS ===\n");

  memory::init_memory(
    config.blob + config.virtual_base,
    config.blob,
    config.page_size,
    config.kernel_base,
    config.kernel_size,
  );

  init_drivers();

  loop {}
}

/// @fn init_drivers
/// @brief Initialize drivers.
fn init_drivers() {
  framebuffer::fb_init();
}
