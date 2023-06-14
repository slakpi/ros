//! AArch64 Initialization

pub mod debug;

mod exceptions;
mod mm;
mod peripherals;

use crate::arch::bits;
use crate::debug_print;
use crate::peripherals::{base, memory, mini_uart, soc};
use crate::support::{dtb, range};

/// Basic kernel configuration provided by the bootstrap code. All address are
/// physical.
#[repr(C)]
struct KernelConfig {
  virtual_base: usize,
  page_size: usize,
  blob: usize,
  kernel_base: usize,
  kernel_size: usize,
  kernel_pages_start: usize,
  kernel_pages_size: usize,
}

/// Re-initialization guard.
static mut INITIALIZED: bool = false;

/// Layout of physical memory in the system.
static mut MEM_LAYOUT: memory::MemoryConfig = memory::MemoryConfig::new();

/// Layout of page allocation exclusions. The physical memory occupied by the
/// kernel, for example, cannot be available for memory allocation.
static mut EXCL_LAYOUT: memory::MemoryConfig = memory::MemoryConfig::new();

/// Page size.
static mut PAGE_SIZE: usize = 0;

/// Page shift.
static mut PAGE_SHIFT: usize = 0;

/// Kernel virtual address base.
static mut VIRTUAL_BASE: usize = 0;

/// AArch64 platform configuration.
///
/// # Parameters
///
/// * `config` - The kernel configuration address provided by the bootstrap
///   code.
///
/// # Description
///
///   NOTE: Must only be called once while the kernel is single-threaded.
///
/// Initializes the interrupt table, determines the physical memory layout,
/// initializes the kernel page tables, and builds a list of exclusions to the
/// physical memory layout.
pub fn init(config: usize) {
  unsafe {
    assert!(!INITIALIZED);
    INITIALIZED = true;
  }

  assert!(config != 0);

  let config = unsafe { &*(config as *const KernelConfig) };

  // TODO: 16 KiB and 64 KiB page support.
  assert!(config.page_size == 4096);

  unsafe {
    PAGE_SIZE = config.page_size;
    PAGE_SHIFT = bits::floor_log2(config.page_size);
    VIRTUAL_BASE = config.virtual_base;
  }

  // Calculate the blob address and its size. There is no need to do any real
  // error checking on the size. If the blob is not valid,
  // `init_memory_layout()` will panic. If the blob is an ATAG list, there is no
  // need to include it in the exclusion list as it will be part of the kernel
  // area exclusion.
  let blob_addr = config.virtual_base + config.blob;
  let blob_size = dtb::DtbReader::check_dtb(blob_addr)
    .map_or_else(|_| 0, |size| bits::align_up(size, config.page_size));

  let mut pages_end = config.kernel_pages_start + config.kernel_pages_size;

  // Initialize the exception vectors.
  exceptions::init();

  // Initialize the real SoC memory layout.
  pages_end = init_soc(config.kernel_pages_start, pages_end, blob_addr);

  // Initialize the Mini UART.
  //
  //   TODO: Remove this once the Mini UART is able to configure itself using
  //         the DTB.
  base::set_peripheral_base_addr(config.virtual_base + 0x7e000000);
  mini_uart::init();

  debug_print!("=== ROS (AArch64) ===\n");

  // Now initialize the physical memory layout.
  pages_end = init_memory_layout(config.kernel_pages_start, pages_end, blob_addr);

  // Initialize the page allocation exclusions.
  init_exclusions(pages_end, config.blob, blob_size);
}

/// Get the physical memory layout.
///
/// # Description
///
///   NOTE: The interface guarantees read-only access outside of the module and
///         one-time initialization is assumed. Therefore, read access is safe
///         and sound.
pub fn get_memory_layout() -> &'static memory::MemoryConfig {
  unsafe { &MEM_LAYOUT }
}

/// Get the page allocation exclusion list.
///
/// # Description
///
///   NOTE: The interface guarantees read-only access outside of the module and
///         one-time initialization is assumed. Therefore, read access is safe
///         and sound.
pub fn get_exclusion_layout() -> &'static memory::MemoryConfig {
  unsafe { &EXCL_LAYOUT }
}

/// Get the page size.
///
/// # Description
///
///   NOTE: The interface guarantees read-only access outside of the module and
///         one-time initialization is assumed. Therefore, read access is safe
///         and sound.
pub fn get_page_size() -> usize {
  unsafe { PAGE_SIZE }
}

/// Get the page shift.
///
/// # Description
///
///   NOTE: The interface guarantees read-only access outside of the module and
///         one-time initialization is assumed. Therefore, read access is safe
///         and sound.
pub fn get_page_shift() -> usize {
  unsafe { PAGE_SHIFT }
}

/// Get the kernel virtual base address.
///
/// # Description
///
///   NOTE: The interface guarantees read-only access outside of the module and
///         one-time initialization is assumed. Therefore, read access is safe
///         and sound.
pub fn get_kernel_virtual_base() -> usize {
  unsafe { VIRTUAL_BASE }
}

/// Initialize the SoC memory layout.
///
/// # Parameters
///
/// * `pages_start` - The start of the kernel's page tables.
/// * `pages_end` - The end of the kernel's page tables.
/// * `blob_addr` - The ATAGs or DTB blob address.
///
/// # Returns
///
/// The new end of the kernel page tables.
fn init_soc(pages_start: usize, pages_end: usize, blob_addr: usize) -> usize {
  let soc_layout = soc::get_soc_memory_layout(blob_addr).unwrap();
  peripherals::init(
    get_kernel_virtual_base(),
    pages_start,
    pages_end,
    &soc_layout,
  )
}

/// Initialize the physical memory layout.
///
/// # Parameters
///
/// * `pages_start` - The start of the kernel's page tables.
/// * `pages_end` - The end of the kernel's page tables.
/// * `blob_addr` - The ATAGs or DTB blob address.
///
/// # Returns
///
/// The new end of the kernel page tables.
fn init_memory_layout(pages_start: usize, pages_end: usize, blob_addr: usize) -> usize {
  let mem_layout = memory::get_memory_layout(blob_addr).unwrap();
  let pages_end = mm::init(
    get_kernel_virtual_base(),
    pages_start,
    pages_end,
    &mem_layout,
  );

  unsafe {
    MEM_LAYOUT = mem_layout;
  }

  pages_end
}

/// Initialize the physical memory exclusion list.
///
/// # Parameters
///
/// * `kernel_size` - The size of the kernel area.
/// * `blob_addr` - The ATAG or DTB blob address.
/// * `blob_size` - The ATAG or DTB blob size.
///
/// # Description
///
/// The kernel area is assumed to start at address 0.
fn init_exclusions(kernel_size: usize, blob_addr: usize, blob_size: usize) {
  let mut excl_layout = memory::MemoryConfig::new();

  excl_layout.insert_range(range::Range {
    base: 0,
    size: kernel_size,
  });

  excl_layout.insert_range(range::Range {
    base: blob_addr,
    size: blob_size,
  });

  excl_layout.trim_ranges();

  unsafe {
    EXCL_LAYOUT = excl_layout;
  }
}
