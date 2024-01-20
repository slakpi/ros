//! ARMv7a Architecture

pub mod debug;
pub mod exceptions;
pub mod mm;
pub mod task;

use crate::debug_print;
use crate::peripherals::{base, memory, mini_uart, soc};
use crate::support::{bits, dtb, range};

/// Reserve the upper 128 MiB of the kernel segment for the high memory area.
const HIGH_MEM_SIZE: usize = 128 * 1024 * 1024;

/// Base address for long-term mappings made by drivers.
const DRIVER_VIRTUAL_BASE: usize = 0xf820_0000;

/// Basic kernel configuration provided by the start code. All address are
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
  vm_split: usize,
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
/// * `config` - The kernel configuration address provided by the start code.
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
  // `init_physical_memory_mappings()` will panic. If the blob is an ATAG list,
  // there is no need to include it in the exclusion list as it will be part of
  // the kernel area exclusion.
  let blob_addr = config.virtual_base + config.blob;
  let blob_size = dtb::DtbReader::check_dtb(blob_addr)
    .map_or_else(|_| 0, |size| bits::align_up(size, config.page_size));

  let mut pages_end = config.kernel_pages_start + config.kernel_pages_size;

  // Initialize the SoC memory mappings.
  //
  //   TODO: Eventually this can be replaced by drivers mapping memory on
  //         demand. For now, just tell the peripherals they are mapped to the
  //         beginning of the driver mapping area.
  pages_end = init_soc_mappings(config.kernel_pages_start, pages_end, blob_addr);
  base::set_peripheral_base_addr(DRIVER_VIRTUAL_BASE);
  mini_uart::init();

  debug_print!("=== ROS (ARMv7 32-bit) ===\n");

  // Initialize the physical memory mappings.
  pages_end = init_physical_memory_mappings(config.kernel_pages_start, pages_end, blob_addr);

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
/// Returns the maximum physical address.
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
/// # Description
///
///   TODO: Eventually this will be replaced by the drivers mapping memory on
///         demand.
///
/// # Returns
///
/// The new end of the kernel page tables.
fn init_soc_mappings(pages_start: usize, pages_end: usize, blob_addr: usize) -> usize {
  let soc_layout = soc::get_soc_memory_layout(blob_addr).unwrap();
  let virtual_base = get_kernel_virtual_base();
  let page_size = get_page_size();
  let mut pages_end = pages_end;
  let mut driver_base = DRIVER_VIRTUAL_BASE;

  for mapping in soc_layout.get_mappings() {
    pages_end = mm::map_memory(
      virtual_base,
      pages_start,
      pages_end,
      driver_base,
      mapping.cpu_base,
      mapping.size,
      true,
    );

    driver_base += bits::align_up(mapping.size, page_size);
  }

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
fn init_physical_memory_mappings(pages_start: usize, pages_end: usize, blob_addr: usize) -> usize {
  // Get the physical memory layout from the blob, then exclude kernel segment.
  // No physical memory beyond the split can be used.
  let mut mem_layout = memory::get_memory_layout(blob_addr).unwrap();
  let virt_base = get_kernel_virtual_base();
  let excl = range::Range {
    base: virt_base,
    size: usize::MAX - virt_base + 1,
  };

  mem_layout.exclude_range(&excl);

  let pages_end = init_kernel_memory_map(virt_base, pages_start, pages_end, &mem_layout);

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
  let excl_layout = memory::MemoryConfig::new_with_ranges(&[
    range::Range {
      base: 0,
      size: kernel_size,
    },
    range::Range {
      base: blob_addr,
      size: blob_size,
    },
  ]);

  unsafe {
    EXCL_LAYOUT = excl_layout;
  }
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
/// The canonical 32-bit 2/2 and 3/1 virtual address space layouts suppored by
/// the kernel:
///
///   +-----------------+ 0xffff_ffff       +-----------------+ 0xffff_ffff
///   |                 |                   |                 |
///   |                 |                   | Kernel Segment  | 1 GiB
///   | Kernel Segment  | 2 GiB             |                 |
///   |                 |                   +-----------------+ 0xc000_0000
///   |                 |                   |                 |
///   +-----------------+ 0x8000_0000       |                 |
///   |                 |                   |                 |
///   |                 |                   | User Segment    | 3 GiB
///   | User Segment    | 2 GiB             |                 |
///   |                 |                   |                 |
///   |                 |                   |                 |
///   +-----------------+ 0x0000_0000       +-----------------+ 0x0000_0000
///
/// Not all ARMv7a CPUs support the Large Physical Address Extensions required
/// for the 3/1 split.
///
/// Kernel segment layout:
///
///   +-----------------+ 0xffff_ffff    -+
///   | / / / / / / / / |                 |
///   |.................| 0xffff_2000     |
///   | Exception Stubs |                 |
///   |.................| 0xffff_1000     |
///   | Vectors         |                 |
///   |.................| 0xffff_0000     +- High Memory
///   |                 |                 |
///   | Driver Mappings |                 |
///   |                 |                 |
///   |.................| 0xf820_0000     |
///   | Temp Mappings   |                 |
///   +-----------------+ 0xf800_0000    -+
///   |                 |                 |
///  ...               ...                |
///   |                 |                 |
///   | Fixed Mappings  |                 +- Low Memory
///   |                 |                 |
///  ...               ...                |
///   |                 | 0xc000_0000 or  |
///   +-----------------+ 0x8000_0000    -+
///
/// The kernel's high memory area occupies the top 128 MiB of the kernel
/// segment. The temp mappings area is reserved for thread-local temporary
/// mappings through `kernel_map_local`. The driver mappings area is reserved
/// for long-term mapping of memory areas that are not mapped in low memory. The
/// exception vectors and stubs areas are simply mappings from the pages in the
/// kernel that contain the vectors and stubs to the high vectors area.
///
///   TODO: What about kernel thread stacks? Presumably these could be part of
///         the task structures allocated in the low memory area, maybe?
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
  // Mask off the physical address range that cannot be mapped into the kernel
  // segment. The amount of physical memory that can be mapped into the kernel
  // segment is the size of the kernel segment minus the high memory area size.
  let base = usize::MAX - virtual_base - HIGH_MEM_SIZE + 1;
  let size = usize::MAX - base + 1;
  let high_mem = range::Range { base, size };

  let mut low_mem = *mem_layout;
  low_mem.exclude_range(&high_mem);

  let mut pages_end = pages_end;

  for range in low_mem.get_ranges() {
    pages_end = mm::direct_map_memory(
      virtual_base,
      pages_start,
      pages_end,
      range.base,
      range.size,
      false,
    );
  }

  pages_end
}
