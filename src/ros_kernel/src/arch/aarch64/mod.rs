//! AArch64 Architecture

pub mod debug;
pub mod exceptions;
pub mod memory;
pub mod mm;
pub mod sync;
pub mod task;

use crate::arch::arm::{cpu, soc};
use crate::debug_print;
use crate::peripherals::{base, mini_uart};
use crate::support::{bits, dtb, range};
use core::ptr;

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
  kernel_stack_list: usize,
  kernel_stack_pages: usize,
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

/// Kernel base address.
static mut KERNEL_BASE: usize = 0;

/// Kernel virtual address base.
static mut VIRTUAL_BASE: usize = 0;

/// Max physical address.
static mut MAX_PHYSICAL_ADDRESS: usize = 0;

/// CPU configuration.
static mut CPU_CONFIG: cpu::CpuConfig = cpu::CpuConfig::new();

/// Kernel stack list base address.
static mut KERNEL_STACK_LIST: usize = 0;

/// Number of pages to allocate for temporary kernel stacks.
static mut KERNEL_STACK_PAGES: usize = 0;

/// AArch64 platform configuration.
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

  // TODO: 16 KiB and 64 KiB page support.
  assert!(config.page_size == 4096);

  // Calculate the blob address and its size. There is no need to do any real
  // error checking on the size. If the blob is not valid,
  // `init_physical_memory_mappings()` will panic. If the blob is an ATAG list,
  // there is no need to include it in the exclusion list as it will be part of
  // the kernel area exclusion.
  let blob_addr = config.virtual_base + config.blob;
  let blob_size = dtb::DtbReader::check_dtb(blob_addr)
    .map_or_else(|_| 0, |size| bits::align_up(size, config.page_size));

  unsafe {
    PAGE_SIZE = config.page_size;
    PAGE_SHIFT = bits::floor_log2(config.page_size);
    KERNEL_BASE = config.kernel_base;
    VIRTUAL_BASE = config.virtual_base;
    MAX_PHYSICAL_ADDRESS = !VIRTUAL_BASE;
    KERNEL_STACK_LIST = config.kernel_stack_list;
    KERNEL_STACK_PAGES = config.kernel_stack_pages;
  }

  // Initialize the CPU information.
  init_cpu_configuration(blob_addr);

  let mut pages_end = config.kernel_pages_start + config.kernel_pages_size;

  // Initialize the SoC memory mappings.
  //
  //   TODO: Eventually this can be replaced by drivers mapping memory on
  //         demand. For now, since we are just directly mapping, use the
  //         default location of the Broadcom SoC on a Raspberry Pi 2 and 3.
  pages_end = init_soc_mappings(config.kernel_pages_start, pages_end, blob_addr);
  base::set_peripheral_base_addr(config.virtual_base + 0x3f00_0000);
  mini_uart::init();

  debug_print!("=== ROS (AArch64) ===\n");

  // Now initialize the physical memory mappings.
  pages_end = init_physical_memory_mappings(config.kernel_pages_start, pages_end, blob_addr);

  // Initialize the page allocation exclusions.
  init_exclusions(pages_end, config.blob, blob_size);
}

pub fn init_secondary_cores() {
  let cpu_config = get_cpu_config();
  let kernel_base = get_kernel_base();
  let virt_base = get_kernel_virtual_base();
  let kernel_stack_list = get_kernel_stack_list();
  let kernel_stack_pages = get_kernel_stack_pages();
  let total_pages = cpu_config.len() * kernel_stack_pages;
  let page_shift = get_page_shift();
  let id_shift = bits::floor_log2((usize::BITS as usize) / 8);
  
  // We have to successfully allocate the stack pages to continue. We can ignore
  // the zone; we are never going to deallocate these pages.
  let (stack_base, stack_pages, _) = crate::mm::kernel_allocate(total_pages).unwrap();
  assert!(stack_pages == total_pages);

  for core in &cpu_config.cores()[1..] {
    unsafe {
      let ptr = (kernel_stack_list + (core.get_id() << id_shift) + virt_base) as *mut usize;
      *ptr = stack_base + (((core.get_id() + 1) * kernel_stack_pages) << page_shift) + virt_base;

      let ptr = (core.get_cpu_release_addr() + virt_base) as *mut usize;
      *ptr = kernel_base; 
    }
  }
}

/// Get the physical memory layout.
///
/// # Description
///
///   NOTE: The interface guarantees read-only access outside of the module and
///         one-time initialization is assumed.
pub fn get_memory_layout() -> &'static memory::MemoryConfig {
  unsafe { ptr::addr_of!(MEM_LAYOUT).as_ref().unwrap() }
}

/// Get the page allocation exclusion list.
///
/// # Description
///
///   NOTE: The interface guarantees read-only access outside of the module and
///         one-time initialization is assumed.
pub fn get_exclusion_layout() -> &'static memory::MemoryConfig {
  unsafe { ptr::addr_of!(EXCL_LAYOUT).as_ref().unwrap() }
}

/// Get the page size.
///
/// # Description
///
///   NOTE: The interface guarantees read-only access outside of the module and
///         one-time initialization is assumed.
pub fn get_page_size() -> usize {
  unsafe { PAGE_SIZE }
}

/// Get the page shift.
///
/// # Description
///
///   NOTE: The interface guarantees read-only access outside of the module and
///         one-time initialization is assumed.
pub fn get_page_shift() -> usize {
  unsafe { PAGE_SHIFT }
}

/// Get the kernel base address.
///
/// # Description
///
///   NOTE: The interface guarantees read-only access outside of the module and
///         one-time initialization is assumed.
pub fn get_kernel_base() -> usize {
  unsafe { KERNEL_BASE }
}

/// Get the kernel virtual base address.
///
/// # Description
///
///   NOTE: The interface guarantees read-only access outside of the module and
///         one-time initialization is assumed.
pub fn get_kernel_virtual_base() -> usize {
  unsafe { VIRTUAL_BASE }
}

/// Get the maximum physical address allowed.
///
/// # Description
///
///   NOTE: The interface guarantees read-only access outside of the module and
///         one-time initialization is assumed.
///
/// # Returns
///
/// Returns the bitwise NOT of the kernel base address.
pub fn get_max_physical_address() -> usize {
  unsafe { MAX_PHYSICAL_ADDRESS }
}

/// Get the number of cores.
///
/// # Description
///
///   NOTE: The interface guarantees read-only access outside of the module and
///         one-time initialization is assumed.
///
/// # Returns
///
/// Returns the number of cores.
pub fn get_core_count() -> usize {
  unsafe { CPU_CONFIG.len() }
}

/// Get the full CPU configuration.
///
/// # Description
///
///   NOTE: The interface guarantees read-only access outside of the module and
///         one-time initialization is assumed.
///
/// # Returns
///
/// Returns the CPU configuration.
pub fn get_cpu_config() -> &'static cpu::CpuConfig {
  unsafe { ptr::addr_of!(CPU_CONFIG).as_ref().unwrap() }
}

pub fn get_kernel_stack_list() -> usize {
  unsafe { KERNEL_STACK_LIST }
}

pub fn get_kernel_stack_pages() -> usize {
  unsafe { KERNEL_STACK_PAGES }
}

/// Initialize the CPU configuration.
///
/// # Parameters
///
/// * `blob_addr` - The ATAGs or DTB blob address.
fn init_cpu_configuration(blob_addr: usize) {
  unsafe {
    assert!(cpu::get_cpu_config(
      ptr::addr_of_mut!(CPU_CONFIG).as_mut().unwrap(),
      blob_addr
    ));
  }
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
  let mut pages_end = pages_end;

  for mapping in soc_layout.get_mappings() {
    pages_end = mm::direct_map_memory(
      virtual_base,
      pages_start,
      pages_end,
      mapping.cpu_base,
      mapping.size,
      true,
    );
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
  unsafe {
    assert!(memory::get_memory_layout(
      ptr::addr_of_mut!(MEM_LAYOUT).as_mut().unwrap(),
      blob_addr
    ));
  }

  let pages_end = init_kernel_memory_map(
    get_kernel_virtual_base(),
    pages_start,
    pages_end,
    get_memory_layout(),
  );

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
/// * `pages_start` - The address of the kernel's Level 1 page table.
/// * `pages_end` - The start of available memory for new page tables.
/// * `mem_layout` - The physical memory layout.
///
/// # Description
///
/// Directly maps all physical memory ranges into the kernel's virtual address
/// space.
///
/// The canonical 64-bit virtual address space layout:
///
///     +-----------------+ 0xffff_ffff_ffff_ffff
///     |                 |
///     | Kernel Segment  | 256 TiB
///     |                 |
///     +-----------------+ 0xffff_0000_0000_0000
///     | / / / / / / / / |
///     | / / / / / / / / |
///     | / / / / / / / / | 16,776,704 TiB of unused address space
///     | / / / / / / / / |
///     | / / / / / / / / |
///     +-----------------+ 0x0000_ffff_ffff_ffff
///     |                 |
///     | User Segment    | 256 TiB
///     |                 |
///     +-----------------+ 0x0000_0000_0000_0000
///
/// This layout allows mapping up to 256 TiB of physical memory into the
/// kernel's address space using a fixed, direct mapping.
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
  let mut pages_end = pages_end;

  for range in mem_layout.get_ranges() {
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
