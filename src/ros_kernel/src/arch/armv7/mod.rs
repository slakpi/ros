//! ARMv7a Architecture

pub mod debug;
pub mod exceptions;
pub mod mm;
pub mod peripherals;
pub mod task;

use crate::peripherals::memory;
use crate::support::{bits, dtb};

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

/// Layout of memory exclusions for the kernel area and DTB, if present.
static mut EXCL_LAYOUT: memory::MemoryConfig = memory::MemoryConfig::new();

/// Page size.
static mut PAGE_SIZE: usize = 0;

/// Page shift.
static mut PAGE_SHIFT: usize = 0;

/// Kernel virtual address base.
static mut VIRTUAL_BASE: usize = 0;

/// Max physical address.
static mut MAX_PHYSICAL_ADDRESS: usize = 0;

/// ARMv7a platform configuration.
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
///   NOTE: Assumes 4 KiB pages.
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

  // TODO: 64 KiB page support.
  assert!(config.page_size == 4096);

  unsafe {
    PAGE_SIZE = config.page_size;
    PAGE_SHIFT = bits::floor_log2(config.page_size);
    VIRTUAL_BASE = config.virtual_base;
    MAX_PHYSICAL_ADDRESS = !VIRTUAL_BASE;
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

  // Initialize the real SoC memory layout.
  // pages_end = init_soc(config.kernel_pages_start, pages_end, blob_addr);

  // Initialize the Mini UART.
  //
  //   TODO: Remove this once the Mini UART is able to configure itself using
  //         the DTB.
  // base::set_peripheral_base_addr(config.virtual_base + 0x7e00_0000);
  // mini_uart::init();

  // debug_print!("=== ROS (ARM) ===\n");

  pages_end = init_memory_layout(config.kernel_pages_start, pages_end, blob_addr);
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

/// Get the physical memory exclusion list.
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

/// Get the maximum physical address allowed.
///
/// # Description
///
///   NOTE: The interface guarantees read-only access outside of the module and
///         one-time initialization is assumed. Therefore, read access is safe
///         and sound.
///
/// # Returns
///
/// Returns the bitwise NOT of the kernel base address.
pub fn get_max_physical_address() -> usize {
  unsafe { MAX_PHYSICAL_ADDRESS }
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
  // let soc_layout = soc::get_soc_memory_layout(blob_addr).unwrap();
  // peripherals::init(
  //   get_kernel_virtual_base(),
  //   pages_start,
  //   pages_end,
  //   &soc_layout,
  // )

  pages_end
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
  // let pages_end = init_kernel_memory_map(
  //   get_kernel_virtual_base(),
  //   pages_start,
  //   pages_end,
  //   &mem_layout,
  // );

  unsafe {
    MEM_LAYOUT = mem_layout;
  }

  pages_end
}

/// Initialize kernel memory map.
///
/// # Parameters
///
/// * `virtual_base` - The kernel segment base address.
/// * `pages_start` - The address of the kernel's 16 KiB Level 1 page table.
/// * `pages_end` - The start of available memory for new page tables.
/// * `mem_layout` - The physical memory layout.
///
/// # Description
///
/// The canonical 32-bit 3:1 virtual address space layout:
///
///   +-----------------+ 0xffff_ffff
///   |                 |
///   | Kernel Segment  | 1 GiB
///   |                 |
///   +-----------------+ 0xc000_0000
///   |                 |
///   |                 |
///   |                 |
///   |                 |
///   | User Segment    | 3 GiB
///   |                 |
///   |                 |
///   |                 |
///   |                 |
///   +-----------------+ 0x0000_0000
///
/// This, of course, means the user processes are limited to accessing 3 GiB of
/// physical memory regardless of the actual amount of physical memory.
///
/// Furthermore, the kernel segment can only directly map at most 1 GiB of
/// physical memory. The canonical way to handle this is to directly map up to
/// 896 MiB of "low memory" into the kernel segment, then use the remaining
/// 128 MiB of "high memory" for things and stuff.
///
/// Refer to `kernel_map_page_local()` and `kernel_unmap_page_local()` for a
/// description of the temporary mappings area.
///
/// Kernel segment layout:
///
///   +-----------------+ 0xffff_ffff    -+
///   | / / / / / / / / |                 |
///   |.................| 0xffff_1000     |
///   | Vectors         |                 |
///   |.................| 0xffff_0000     |
///   |                 |                 +- High Memory
///   | ???             |                 |
///   |                 |                 |
///   |.................| 0xf800_1000     |
///   | Temp Mappings   |                 |
///   +-----------------+ 0xf800_0000    -+
///   |                 |                 |
///   |                 |                 |
///   | Fixed Mappings  | 896 MiB         +- Low Memory
///   |                 |                 |
///   |                 |                 |
///   +-----------------+ 0xc000_0000    -+
///
/// # Returns
///
/// The new end of the page table area.
fn init_kernel_memory_map(
  virtual_base: usize,
  pages_start: usize,
  pages_end: usize,
  mem_layout: &memory::MemoryConfig,
) -> usize {
  pages_end
}
