//! ARMv7a peripherals management.

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
///     TODO: I have no idea how 0x7e000000 will be mapped into the kernel's
///           virtual address space. ¯\_(ツ)_/¯
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
  pages_end
}
