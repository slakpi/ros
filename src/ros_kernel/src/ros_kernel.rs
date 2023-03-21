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
  dbg_print!("Kernel Panic: {}\n", info);
  loop {}
}

/// Kernel stub.
///
/// # Parameters
///
/// * `init` - The kernel configuration provided by the bootstrap code.
///
/// # Returns
///
/// Does not return.
#[no_mangle]
extern "C" fn ros_kernel(init: KernelConfig) -> ! {
  let mut pages_end = init.kernel_pages_start + init.kernel_pages_size;

  init_exceptions();

  pages_end = init_peripherals(
    init.virtual_base,
    init.peripheral_base,
    init.peripheral_block_size,
    init.kernel_pages_start,
    pages_end,
  );

  _ = init_memory(init.virtual_base, init.blob, init.kernel_pages_start, pages_end);
  
  init_drivers();
  
  loop {}
}

/// Initialize architecture-dependent exception vectors.
fn init_exceptions() {
  exceptions::init_exception_vectors();
}

/// Initialize peripheral devices.
///
///     TODO: At some point in the long and distant future, this information
///           should be derived from the DTB and fed to drivers that map the
///           memory.
///
/// # Parameters
///
/// * `virtual_base` - The kernel segment base address.
/// * `peripheral_base` - The base physical address of the peripherals.
/// * `peripheral_block_size` - The size of the peripheral memory.
/// * `pages_start` - The address of the kernel's Level 1 page table.
/// * `pages_end` - The start of available memory for new pages.
///
/// # Description
///
/// Passes the peripheral memory range to the memory manager. The memory manager
/// directly maps the device memory into the virtual address space. Once the
/// device memory is mapped, the function initializes the mini-UART to output
/// diagnostic messages.
///
/// # Returns
///
/// The new end of the page table area.
fn init_peripherals(
  virtual_base: usize,
  peripheral_base: usize,
  peripheral_block_size: usize,
  pages_start: usize,
  pages_end: usize
) -> usize {
  let mut mem_config = memory::MemoryConfig::new();

  mem_config.insert_range(memory::MemoryRange {
    base: peripheral_base,
    size: peripheral_block_size,
    device: true,
  });

  let pages_end = mm::direct_map_memory(virtual_base, pages_start, pages_end, &mem_config);

  base::set_peripheral_base_addr(peripheral_base + virtual_base);
  mini_uart::init_uart();

  dbg_print!("=== ROS ===\n");
  dbg_print!(
    "Initialized peripheral device memory: {:#x} - {:#x}\n",
    peripheral_base,
    peripheral_base + peripheral_block_size - 1,
  );

  pages_end
}

/// Initialize memory.
///
/// # Parameters
///
/// * `init` - The kernel configuration provided by the bootstrap code.
///
/// # Description
///
/// Attempts to retrieve the memory layout from ATAGs or a DTB, and passes the
/// layout on to the memory manager. The memory manager directly maps the
/// physical memory into the virtual address space as appropriate for the
/// architecture.
fn init_memory(virtual_base: usize, blob: usize, pages_start: usize, pages_end: usize) -> usize {
  let mem_config = memory::get_memory_layout(virtual_base + blob).unwrap();
  let pages_end = mm::direct_map_memory(virtual_base, pages_start, pages_end, &mem_config);

  dbg_print!("Initialized physical memory at:\n");

  for range in mem_config.get_ranges() {
    dbg_print!(
      "  {:#x} - {:#x}\n",
      range.base,
      range.base + range.size - 1,
    );
  }

  pages_end
}

/// Initialize drivers.
fn init_drivers() {
  framebuffer::fb_init();
}
