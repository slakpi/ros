use super::drivers::video::framebuffer;
use super::peripherals::{base, memory, mini_uart};
use core::panic::PanicInfo;

/// @fn panic(_info: &PanicInfo) -> !
/// @brief   Panic handler.
/// @param[in] info The panic info.
/// @returns Does not return.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
  loop {}
}

/// @pub extern "C" fn ros_kernel(
///        blob: u32,
///        peripheral_base: u32,
///        page_size: u32,
///        kernel_base: u32,
///        kernel_size: u32,
///      ) -> ! {
/// @brief   Kernel stub.
/// @details TODO: The page size comes across in ATAGs. Should research if it
///          also comes across in the device tree and consider not passing it in
///          as a parameter here.
/// @param[in] blob            ATAG or Device Tree blob.
/// @param[in] peripheral_base The peripheral base address.
/// @param[in] page_size       Configured memory page size.
/// @param[in] kernel_base     Base address of the kernel.
/// @param[in] kernel_size     Kernel image size.
/// @returns Does not return
#[no_mangle]
pub extern "C" fn ros_kernel(
  blob: u32,
  peripheral_base: u32,
  page_size: u32,
  kernel_base: u32,
  kernel_size: u32,
) -> ! {
  base::set_peripheral_base_addr(peripheral_base as usize);
  mini_uart::init_uart();
  memory::init_memory(
    blob as usize,
    page_size as usize,
    kernel_base as usize,
    kernel_size as usize,
  );
  init_drivers();
  loop {}
}

/// @fn init_drivers()
/// @brief Initialize drivers.
fn init_drivers() {
  framebuffer::fb_init();
}
