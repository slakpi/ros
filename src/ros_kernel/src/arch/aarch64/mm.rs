use crate::peripherals::memory;

/// Initialize the AArch64 page tables for the kernel. The canonical 64-bit
/// virtual address space layout for a process looks like:
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
/// Level 1 table for a total of 520 KiB. That could be reduced to 8 KiB and
/// eliminate a second level of translation by using one Level 2 table with 128
/// 1 GiB entries and one Level 1 table.
///
/// The entire 128 TiB address space could be mapped using 256 Level 2 tables
/// and one Level 1 table for a total of 1 MiB.
///
/// This is separate from actually allocating physical pages to the kernel. It
/// just means no page table operations need to be performed when allocating a
/// page to the kernel.
pub fn init_memory(virtual_base: usize, pages_start: usize, mem_config: &memory::MemoryConfig) {

}