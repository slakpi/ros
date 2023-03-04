use crate::peripherals::memory;

/// Initialize the AArch64 page tables for the kernel.
///
/// # Parameters
///
/// * `virtual_base` - The kernel segment base address.
/// * `pages_start` - The address of the kernel's Level 1 page table.
/// * `mem_config` - 
///
/// # Details
///
///     TODO: For now, memory management will just assume 4 KiB pages. The
///           bootstrap code will have already configured TCR_EL1 with 4 KiB
///           granules.
///
/// The canonical 64-bit virtual address space layout for a process looks like:
///
///     +-----------------+ 0xffff_ffff_ffff_ffff
///     |                 |
///     | Kernel Segment  | 128 TiB
///     |                 |
///     +-----------------+ 0xffff_8000_0000_0000
///     | / / / / / / / / |
///     | / / / / / / / / | 16,776,960 TiB of unused address space
///     | / / / / / / / / |
///     +-----------------+ 0x0000_8000_0000_0000
///     |                 |
///     | User Segment    | 128 TiB
///     |                 |
///     +-----------------+ 0x0000_0000_0000_0000
///
/// AArch64 provides two independent registers for address translation so that
/// the kernel does not need to be mapped into the translation tables for every
/// process. The most-significant bit selects the register used for translation.
///
/// AArch64 provides four levels of address space translation. With 4 KiB pages,
/// the page tables can address 256 TiB of memory:
///
///     Level 1   ->    Level 2   ->    Level 3   ->    Level 4
///     Covers          Covers          Covers          Covers
///     256 TiB         512 GiB         1 GiB           2 MiB
///
/// Each page table itself is 4 KiB (512 entries, each 64-bits).
///
/// AArch64 allows skipping lower levels of translation. Each Level 2 entry can
/// point to a Level 3 table OR a 1 GiB block of memory. Each Level 3 entry can
/// point to a Level 4 table OR a 2 MiB block of memory.
///
/// Currently, a single kernel is not expected to deal with anywhere near 128
/// TiB of physical memory, so it is feasible to directly map the entire
/// physical address space into the kernel segment.
///
/// Sticking with 4 KiB pages and skipping Level 4 translation, a 128 GiB
/// address space would require 128 Level 3 tables, one Level 2 table, and the
/// Level 1 table for a total of 520 KiB. That can be reduced to 8 KiB and
/// eliminate Level 3 translation by using one Level 2 table with 128 1 GiB
/// entries and one Level 1 table.
///
/// A mixture will be used here since the memory layout provided may have blocks
/// that are non-integer multiples of 1 GiB.
///
/// This mapping is separate from allocating pages to the kernel.
pub fn init_memory(virtual_base: usize, pages_start: usize, mem_config: &memory::MemoryConfig) {

}
