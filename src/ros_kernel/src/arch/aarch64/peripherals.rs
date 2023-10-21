//! AArch64 Peripherals Management

use super::mm;
use crate::peripherals::soc;

/// Initialize peripheral mappings.
///
/// # Parameters
///
/// * `virtual_base` - The kernel segment base address.
/// * `pages_start` - The address of the kernel's Level 1 page table.
/// * `pages_end` - The start of available memory for new pages.
/// * `soc_layout` - The mapping of SoC memory to ARM CPU memory.
///
/// # Details
///
/// The `soc` section of the DTB specifies the mappings from SoC addresses to
/// ARM CPU physical addresses. The SoC address ranges will map from the
/// kernel's address space to the physical address. Peripheral devices will use
/// virtual addresses to reference hardware.
///
/// For AArch64, the kernel will simply direct map the ARM CPU physical address
/// into the kernel's address space.
///
/// # Returns
///
/// The new end of the page table area.
pub fn init(
  virtual_base: usize,
  pages_start: usize,
  pages_end: usize,
  soc_layout: &soc::SocConfig,
) -> usize {
  let mut pages_end = pages_end;

  for mapping in soc_layout.get_mappings() {
    pages_end = mm::direct_map_memory(
      virtual_base,
      pages_start,
      pages_end,
      mapping.cpu_base,
      mapping.size,
      true,
    )
  }

  pages_end
}
