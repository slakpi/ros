//! ARMv7a memory management.

/// Initialize memory.
///
/// # Parameters
///
/// * `virtual_base` - The kernel segment base address.
/// * `blob` - ATAG or DTB blob.
/// * `pages_start` - The address of the kernel's Level 1 page table.
/// * `pages_end` - The start of available memory for new pages.
///
/// # Description
///
/// Attempts to retrieve the memory layout from ATAGs or a DTB, and passes the
/// layout on to the memory manager. The memory manager directly maps the
/// physical memory into the virtual address space as appropriate for the
/// architecture.
pub fn init(virtual_base: usize, blob: usize, pages_start: usize, pages_end: usize) -> usize {
  pages_end
}

/// Initialize the ARMv7 page tables for the kernel. The canonical 32-bit 3:1
/// virtual address space layout for a process looks like:
///
///   +-----------------+ 0xffff_ffff
///   |                 |
///   | Kernel Segment  | 1 GiB
///   |                 |
///   +-----------------+ 0xc000_0000
///   |                 |
///   | User Segment    | 3 GiB
///   |                 |
///   +-----------------+ 0x0000_0000
///
pub fn direct_map_memory(
  virtual_base: usize,
  pages_start: usize,
  pages_end: usize,
  base: usize,
  size: usize,
  device: bool,
) -> usize {
  pages_end
}
