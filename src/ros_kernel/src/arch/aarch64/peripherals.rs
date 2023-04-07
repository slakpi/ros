//! AArch64 peripherals management.

use super::mm;
use crate::peripherals::{base, soc};

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
/// For example, the BCM2710 uses 0x7e000000 as the base address for
/// peripherals. The Raspberry Pi 3 maps this address to the ARM CPU address
/// 0x3f000000. The page tables will map 0xffff_8000_7e00_0000 => 0x3f000000.
///
///     TODO: Eventually, this function should be completely unaware of the SoC
///           base address. Device addresses in the DTB should be used instead.
///           For now, it will just assume the BCM271x base address.
///
///           This does, however, make the kernel flexible enough to configure
///           itself for Raspberry Pi 2/3 or Raspberry Pi 4 at runtime.
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
    pages_end = mm::map_memory(
      virtual_base,
      pages_start,
      pages_end,
      mapping.soc_base,
      mapping.cpu_base,
      mapping.size,
      true,
    )
  }

  base::set_peripheral_base_addr(virtual_base + 0x7e000000);

  pages_end
}
