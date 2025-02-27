//! AArch64 Architecture

pub mod debug;
pub mod exceptions;
pub mod memory;
pub mod mm;
pub mod sync;
pub mod task;

use crate::arch::arm::{cpu, soc};
use crate::mm::{MappingStrategy, TableAllocator};
use crate::peripherals::{base, mini_uart};
use crate::support::{bits, dtb, range};
use core::ptr;

/// Basic kernel configuration provided by the start code. All address are
/// physical.
#[repr(C)]
#[derive(Copy, Clone)]
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
  primary_stack_start: usize,
}

/// The base virtual address of the page directory.
const PAGE_DIRECTORY_VIRTUAL_BASE: usize = 0xffff_fe00_0000_0000;

/// The size of the virtual area reserved for the page directory (2 TiB).
const PAGE_DIRECTORY_VIRTUAL_SIZE: usize = 0x200_0000_0000;

/// Re-initialization guard.
static mut INITIALIZED: bool = false;

static mut KERNEL_CONFIG: KernelConfig = KernelConfig {
  virtual_base: 0,
  page_size: 0,
  blob: 0,
  kernel_base: 0,
  kernel_size: 0,
  kernel_pages_start: 0,
  kernel_pages_size: 0,
  kernel_stack_list: 0,
  kernel_stack_pages: 0,
  primary_stack_start: 0,
};

/// Layout of physical memory in the system.
static mut MEM_LAYOUT: memory::MemoryConfig = memory::MemoryConfig::new();

/// Layout of page allocation exclusions. The physical memory occupied by the
/// kernel, for example, cannot be available for memory allocation.
static mut EXCL_LAYOUT: memory::MemoryConfig = memory::MemoryConfig::new();

/// Page shift.
static mut PAGE_SHIFT: usize = 0;

/// Max physical address.
static mut MAX_PHYSICAL_ADDRESS: usize = 0;

/// CPU configuration.
static mut CPU_CONFIG: cpu::CpuConfig = cpu::CpuConfig::new();

/// The base virtual address of the kernel thread stack area.
static mut KERNEL_THREAD_STACK_VIRTUAL_BASE: usize = 0;

/// The base virtual address of the kernel ISR stack area.
static mut KERNEL_ISR_STACK_VIRTUAL_BASE: usize = 0;

/// Simple allocator for use before the kernel's page allocators are
/// initialized.
struct LinearTableAllocator {
  next_table: usize,
}

impl LinearTableAllocator {
  /// Construct a new linear table allocator.
  ///
  /// # Parameters
  ///
  /// * `next_table` - The next available physical address for new tables.
  pub fn new(next_table: usize) -> Self {
    LinearTableAllocator { next_table }
  }

  /// Get the physical address of the next table.
  fn get_next_table(&self) -> usize {
    self.next_table
  }
}

impl TableAllocator for LinearTableAllocator {
  /// Allocate a new table page.
  ///
  /// # Description
  ///
  /// Simply returns the next page address.
  ///
  ///   TODO: The allocator just assumes it is allocating from available memory.
  ///         The allocator really should have an upper bound.
  ///
  /// # Returns
  ///
  /// The physical address of the new table.
  fn alloc_table(&mut self) -> usize {
    let new_table = self.next_table;
    self.next_table += get_page_size();
    new_table
  }
}

/// Dynamic table allocator.
struct DynamicTableAllocator {
  next_table: usize,
  avail_tables: usize,
  zone: usize,
}

impl DynamicTableAllocator {
  /// Construct a new dynamic table allocator.
  pub fn new() -> Self {
    DynamicTableAllocator {
      next_table: 0,
      avail_tables: 0,
      zone: 0,
    }
  }
}

impl TableAllocator for DynamicTableAllocator {
  /// Allocate a new table page.
  ///
  /// # Returns
  ///
  /// The physical address of the new table.
  fn alloc_table(&mut self) -> usize {
    if self.avail_tables == 0 {
      (self.next_table, self.avail_tables, self.zone) = crate::mm::kernel_allocate(4).unwrap();
    }

    assert!(self.avail_tables > 0);

    let new_table = self.next_table;
    self.next_table += get_page_size();
    self.avail_tables -= 1;
    new_table
  }
}

impl Drop for DynamicTableAllocator {
  /// Free unused tables.
  fn drop(&mut self) {
    while self.avail_tables > 0 {
      crate::mm::kernel_free(self.next_table, 1, self.zone);
      self.next_table += get_page_size();
      self.avail_tables -= 1;
    }
  }
}

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
///   NOTE: Assumes the blob is a DTB.
///
/// Initializes the interrupt table, determines the physical memory layout,
/// initializes the kernel page tables, and builds a list of exclusions to the
/// physical memory layout.
pub fn init(config: usize) {
  assert!(get_core_id() == 0);

  unsafe {
    assert!(!INITIALIZED);
    INITIALIZED = true;
  }

  assert!(config != 0);

  let kconfig = unsafe { &*(config as *const KernelConfig) };

  // Calculate the blob address and its size. There is no need to do any real
  // error checking on the size. If the blob is not valid,
  // `init_physical_memory_mappings()` will panic.
  let blob_vaddr = kconfig.virtual_base + kconfig.blob;
  let blob_size = dtb::DtbReader::check_dtb(blob_vaddr)
    .map_or_else(|_| 0, |size| bits::align_up(size, kconfig.page_size));

  // TODO: 16 KiB and 64 KiB page support.
  assert!(kconfig.page_size == 4096);

  unsafe {
    KERNEL_CONFIG = *kconfig;
    PAGE_SHIFT = bits::floor_log2(kconfig.page_size);
    MAX_PHYSICAL_ADDRESS = !KERNEL_CONFIG.virtual_base;
  }

  // Get the CPU configuration from the DTB.
  init_cpu_configuration(blob_vaddr);

  let mut table_allocator =
    LinearTableAllocator::new(kconfig.kernel_pages_start + kconfig.kernel_pages_size);

  // Initialize the SoC memory mappings.
  //
  //   TODO: Eventually this can be replaced by drivers mapping memory on
  //         demand. For now, since we are just directly mapping, use the
  //         default location of the Broadcom SoC on a Raspberry Pi 2 and 3.
  init_soc_mappings(blob_vaddr, kconfig.kernel_pages_start, &mut table_allocator);
  base::set_peripheral_base_addr(kconfig.virtual_base + 0x3f00_0000);
  mini_uart::init();

  // Initialize the physical memory mappings.
  init_physical_memory_mappings(blob_vaddr, kconfig.kernel_pages_start, &mut table_allocator);

  // Initialize the page allocation exclusions. The allocator allocates pages
  // linearly starting from the end of the kernel image, so just use exclude
  // everything from 0 to the next available address.
  init_exclusions(table_allocator.get_next_table(), kconfig.blob, blob_size);
}

/// Initialize any secondary cores. The kernel is considered multi-threaded when
/// this function returns.
pub fn init_multi_core() {
  assert!(get_core_id() == 0);

  let virt_base = get_kernel_virtual_base();
  let kernel_base = get_kernel_base();
  let core_count = get_core_count();
  let cpu_config = get_cpu_config();

  let mut table_allocator = DynamicTableAllocator::new();

  // Initialize the kernel ISR stacks and map them.
  init_kernel_isr_stacks(&mut table_allocator);

  // There are no cores to release.
  if core_count < 2 {
    return;
  }

  // Write the kernel start address to the secondary core release addresses.
  for core in &cpu_config.get_cores()[1..] {
    let ptr = (virt_base + core.get_release_addr()) as *mut usize;
    unsafe {
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
  unsafe { KERNEL_CONFIG.page_size }
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
  unsafe { KERNEL_CONFIG.kernel_base }
}

/// Get the kernel virtual base address.
///
/// # Description
///
///   NOTE: The interface guarantees read-only access outside of the module and
///         one-time initialization is assumed.
pub fn get_kernel_virtual_base() -> usize {
  unsafe { KERNEL_CONFIG.virtual_base }
}

/// Get the maximum physical address allowed.
///
/// # Description
///
///   NOTE: The interface guarantees read-only access outside of the module and
///         one-time initialization is assumed.
pub fn get_max_physical_address() -> usize {
  unsafe { MAX_PHYSICAL_ADDRESS }
}

/// Get the number of cores.
///
/// # Description
///
///   NOTE: The interface guarantees read-only access outside of the module and
///         one-time initialization is assumed.
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
///   TODO: Make an architecture-independent core information struct and replace
///         this with a function that returns the information about a specific
///         core rather than returning the architecture-dependent information.
///
///   TODO: Do not make cpu::CpuConfig public outside of the `arch` module.
pub fn get_cpu_config() -> &'static cpu::CpuConfig {
  unsafe { ptr::addr_of!(CPU_CONFIG).as_ref().unwrap() }
}

/// Get the identifier of the current core.
///
/// # Description
///
///   NOTE: The interface guarantees read-only access outside of the module and
///         one-time initialization is assumed.
pub fn get_core_id() -> usize {
  cpu::get_core_id()
}

/// Get the virtual address of the page directory.
pub fn get_page_directory_virtual_base() -> usize {
  PAGE_DIRECTORY_VIRTUAL_BASE
}

/// Get the size of the virtual area reserved for the page directory.
pub fn get_page_directory_virtual_size() -> usize {
  PAGE_DIRECTORY_VIRTUAL_SIZE
}

/// Get the address of the Level 1 translation table.
///
/// # Description
///
///   NOTE: The interface guarantees read-only access outside of the module and
///         one-time initialization is assumed.
fn get_pages_start() -> usize {
  unsafe { KERNEL_CONFIG.kernel_pages_start }
}

/// Get the starting virtual address of the primary core's ISR stack.
///
/// # Description
///
///   NOTE: The interface guarantees read-only access outside of the module and
///         one-time initialization is assumed.
fn get_primary_stack_start() -> usize {
  unsafe { KERNEL_CONFIG.primary_stack_start }
}

/// Get the physical base address of the list of kernel ISR stack addresses.
///
/// # Description
///
///   NOTE: The interface guarantees read-only access outside of the module and
///         one-time initialization is assumed.
fn get_kernel_stack_list() -> usize {
  unsafe { KERNEL_CONFIG.kernel_stack_list }
}

/// Get the kernel ISR and thread stack size.
///
/// # Description
///
///   NOTE: The interface guarantees read-only access outside of the module and
///         one-time initialization is assumed.
fn get_kernel_stack_pages() -> usize {
  unsafe { KERNEL_CONFIG.kernel_stack_pages }
}

/// Get the base virtual address of the kernel ISR stacks.
///
/// # Description
///
///   NOTE: The interface guarantees read-only access outside of the module and
///         one-time initialization is assumed.
fn get_kernel_isr_stack_virtual_base() -> usize {
  unsafe { KERNEL_ISR_STACK_VIRTUAL_BASE }
}

/// Get the base virtual address of the kernel thread stacks.
///
/// # Description
///
///   NOTE: The interface guarantees read-only access outside of the module and
///         one-time initialization is assumed.
fn get_kernel_thread_stack_virtual_base() -> usize {
  unsafe { KERNEL_THREAD_STACK_VIRTUAL_BASE }
}

/// Initialize the CPU configuration.
///
/// # Parameters
///
/// * `blob_addr` - The DTB blob address.
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
/// * `blob_addr` - The DTB blob address.
/// * `pages_start` - The start of the kernel's page tables.
/// * `allocator` - The table page allocator.
///
/// # Description
///
///   TODO: Eventually this will be replaced by the drivers mapping memory on
///         demand.
fn init_soc_mappings(blob_addr: usize, pages_start: usize, allocator: &mut impl TableAllocator) {
  let soc_layout = soc::get_soc_memory_layout(blob_addr).unwrap();
  let virtual_base = get_kernel_virtual_base();

  for mapping in soc_layout.get_mappings() {
    mm::direct_map_memory(
      virtual_base,
      pages_start,
      mapping.cpu_base,
      mapping.size,
      true,
      allocator,
      MappingStrategy::Compact,
    );
  }
}

/// Initialize the physical memory layout.
///
/// # Parameters
///
/// * `blob_addr` - The DTB blob address.
/// * `pages_start` - The start of the kernel's page tables.
/// * `allocator` - The table page allocator.
///
/// # Description
///
/// Linearly maps physical memory into the kernel address space. Excludes any
/// physical memory that would be in the regions reserved for the page directory
/// and kernel ISR stacks and thread stacks.
fn init_physical_memory_mappings(
  blob_addr: usize,
  pages_start: usize,
  allocator: &mut impl TableAllocator,
) {
  let core_count = get_core_count();
  let kernel_stack_pages = get_kernel_stack_pages();
  let page_shift = get_page_shift();
  let stack_size = (kernel_stack_pages + 1) << page_shift;
  let region_size = stack_size * core_count;
  let virt_base = get_kernel_virtual_base();

  unsafe {
    KERNEL_THREAD_STACK_VIRTUAL_BASE = PAGE_DIRECTORY_VIRTUAL_BASE - region_size;
    KERNEL_ISR_STACK_VIRTUAL_BASE = KERNEL_THREAD_STACK_VIRTUAL_BASE - region_size;
  }

  // Exclude the top 2 TiB plus the kernel ISR stack and thread stack regions.
  let excl = range::Range {
    base: get_kernel_isr_stack_virtual_base() - virt_base,
    size: PAGE_DIRECTORY_VIRTUAL_SIZE + (region_size * 2),
  };

  // Get the physical memory layout and apply the exclusion.
  unsafe {
    let mut mem_layout = ptr::addr_of_mut!(MEM_LAYOUT).as_mut().unwrap();
    assert!(memory::get_memory_layout(&mut mem_layout, blob_addr));
    mem_layout.exclude_range(&excl);
  }

  // Map the remaining physical memory into the kernel address space.
  init_kernel_memory_map(virt_base, get_memory_layout(), pages_start, allocator);
}

/// Initialize the physical memory exclusion list.
///
/// # Parameters
///
/// * `kernel_size` - The size of the kernel area.
/// * `blob_addr` - The DTB blob address.
/// * `blob_size` - The DTB blob size.
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
/// * `mem_layout` - The physical memory layout.
/// * `pages_start` - The address of the kernel's Level 1 page table.
/// * `allocator` - The table page allocator.
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
fn init_kernel_memory_map(
  virtual_base: usize,
  mem_layout: &memory::MemoryConfig,
  pages_start: usize,
  allocator: &mut impl TableAllocator,
) {
  for range in mem_layout.get_ranges() {
    mm::direct_map_memory(
      virtual_base,
      pages_start,
      range.base,
      range.size,
      false,
      allocator,
      MappingStrategy::Compact,
    );
  }
}

/// Allocate pages for the ISR stacks and map them into the ISR stack area.
///
/// # Parameters
///
/// * `allocator` - The table page allocator.
///
/// # Description
///
/// Allocates memory for the core ISR stacks, then maps the stacks into the ISR
/// stack area. The stacks are separated by unmapped guard pages.
fn init_kernel_isr_stacks(allocator: &mut impl TableAllocator) {
  let core_count = get_core_count();
  let cpu_config = get_cpu_config();
  let kernel_base = get_kernel_base();
  let virt_base = get_kernel_virtual_base();
  let page_shift = get_page_shift();
  let kernel_stack_list = get_kernel_stack_list();
  let kernel_stack_pages = get_kernel_stack_pages();
  let stack_size = kernel_stack_pages << page_shift;
  let isr_base = get_kernel_isr_stack_virtual_base();
  let primary_stack_start = get_primary_stack_start();
  let pages_start = get_pages_start();

  // Each stack includes a guard page that will be unmapped from the kernel's
  // virtual address space. If the kernel stack overflows, this will cause an
  // exception.
  let stack_virtual_offset = (kernel_stack_pages + 1) << page_shift;

  // Map the primary core's stack into the ISR area.
  mm::map_memory(
    virt_base,
    pages_start,
    isr_base + (1 << page_shift) - virt_base,
    primary_stack_start - stack_size - virt_base,
    stack_size,
    false,
    allocator,
    MappingStrategy::Granular,
  );

  // Rebase the primary core's stack pointer. The physical memory is the same,
  // the stack pointer just needs to use virtual address in the ISR stack area
  // with the guard page instead of the virtual address in the kernel image.
  unsafe { KERNEL_CONFIG.primary_stack_start = isr_base + stack_virtual_offset };
  cpu::remap_stack(primary_stack_start, isr_base + stack_virtual_offset);

  // Nothing left to do if running with a single core.
  if core_count < 2 {
    return;
  }

  for core in &cpu_config.get_cores()[1..] {
    let core_id = core.get_id();

    // We have to successfully allocate the stack pages to continue. Ideally,
    // `kernel_stack_pages` is a power of 2. If it is, allocating each stack
    // individually eliminates over-allocation.
    //
    // Ignore the zone, we are not going to free these pages.
    let (stack_base, stack_pages, _) = crate::mm::kernel_allocate(kernel_stack_pages).unwrap();
    assert!(stack_pages >= kernel_stack_pages);

    // Calculate the stack address list entry. The entry for Core 0 is left
    // uninitialized.
    //
    //     +---------------------------+ +8 * N
    //     | Core N ISR Stack Address  |
    //    ...                         ...
    //     | Core 3 ISR Stack Address  |
    //     +---------------------------+ +24
    //     | Core 2 ISR Stack Address  |
    //     +---------------------------+ +16
    //     | Core 1 ISR Stack Address  |
    //     +---------------------------+ +8
    //     | / / / / / / / / / / / / / |
    //     +---------------------------+  virt_base + kernel_stack_list
    let addr_offset = core_id << 3;
    let ptr = (virt_base + kernel_stack_list + addr_offset) as *mut usize;

    // Next, calculate the address of the stack for this core and place it in
    // the stack address list.
    //
    //     +---------------------------+ +stack_virtual_offset * N
    //     | Core N ISR Stack          |
    //     +---------------------------+
    //     | / / / / / Guard / / / / / |
    //     +---------------------------+
    //    ...                         ...
    //     +---------------------------+
    //     | Core 2 ISR Stack          |
    //     +---------------------------+
    //     | / / / / / Guard / / / / / |
    //     +---------------------------+ +stack_virtual_offset * 2
    //     | Core 1 ISR Stack          |
    //     +---------------------------+
    //     | / / / / / Guard / / / / / |
    //     +---------------------------+ +stack_virtual_offset
    //     | Core 0 ISR Stack          |
    //     +---------------------------+
    //     | / / / / / Guard / / / / / |
    //     +---------------------------+  virt_base + stack_base
    let stack_virtual_base = isr_base + (stack_virtual_offset * core_id) + (1 << page_shift);
    unsafe {
      *ptr = stack_virtual_base + stack_size;
    }

    // Map the core's stack into the ISR stack area.
    mm::map_memory(
      virt_base,
      pages_start,
      stack_virtual_base - virt_base,
      stack_base,
      stack_size,
      false,
      allocator,
      MappingStrategy::Granular,
    );
  }
}
