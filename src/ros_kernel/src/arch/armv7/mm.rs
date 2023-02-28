use crate::peripherals::memory;

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
pub fn init_memory(virtual_base: usize, pages_start: usize, mem_config: &memory::MemoryConfig) {}
