const MAX_MEM_REGIONS: usize = 16;

/// @struct ROSMemoryRegion
/// @brief Memory region available to the kernel.
/// @var base The base address of the region.
/// @var size The size of the region in bytes.
#[derive(Copy, Clone)]
pub struct ROSMemoryRegion {
  pub base: usize,
  pub size: usize,
}

impl ROSMemoryRegion {
  pub fn new() -> Self {
    ROSMemoryRegion { base: 0, size: 0 }
  }
}

/// @struct ROSKernelInit
/// @brief Initialization parameters provided by the bootloader.
/// @var peripheral_base The base address for peripherals.
/// @var memory_regions  Memory regions available to the kernel.
pub struct ROSKernelInit {
  pub peripheral_base: usize,
  pub memory_regions: [ROSMemoryRegion; MAX_MEM_REGIONS],
}

impl ROSKernelInit {
  pub fn new() -> Self {
    ROSKernelInit {
      peripheral_base: 0x0,
      memory_regions: [ROSMemoryRegion::new(); MAX_MEM_REGIONS],
    }
  }
}
