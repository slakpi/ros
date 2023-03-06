//! ROS Kernel entry point.

use super::drivers::video::framebuffer;
use super::exceptions;
use super::mm;
use super::peripherals::{base, memory, mini_uart};
use crate::dbg_print;
use core::panic::PanicInfo;

/// Basic kernel configuration provided by the bootstrap code. All address are
/// physical.
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

/// Panic handler. Prints out diagnostic information and halts.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
  dbg_print!("Kernel Panic: {}", info);
  loop {}
}

/// Kernel stub.
#[no_mangle]
extern "C" fn ros_kernel(init: KernelConfig) -> ! {
  init_exceptions();
  init_memory(&init);
  init_peripherals(&init);

  dbg_print!("=== ROS ===\n");

  init_drivers();

  loop {}
}

/// Initialize architecture-dependent exception vectors.
fn init_exceptions() {
  exceptions::init_exception_vectors();
}

/// Initialize memory. Attempts to retrieve the memory layout from ATAGs or a
/// DTB, and passes the layout on to the memory manager. Halts the kernel if
/// unable to get the memory layout.
fn init_memory(init: &KernelConfig) {
  // Get the physical memory layout.
  let mut mem_config = memory::get_memory_layout(init.virtual_base + init.blob).unwrap();

  // Add the peripheral memory range to the memory layout marking it as device
  // memory.
  mem_config.insert_range(memory::MemoryRange {
    base: init.peripheral_base,
    size: init.peripheral_block_size,
    device: true,
  });
  mem_config.trim_ranges();

  // Initialize the kernel memory map.
  mm::init_memory(init.virtual_base, init.kernel_pages_start, &mem_config);
}

/// Initialize peripheral devices. TODO: this will be replaced by drivers
/// mapping the memory they need.
fn init_peripherals(init: &KernelConfig) {
  base::set_peripheral_base_addr(init.peripheral_base + init.virtual_base);
  mini_uart::init_uart();
}

/// Initialize drivers.
fn init_drivers() {
  framebuffer::fb_init();
}
