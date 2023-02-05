use crate::peripherals::memory;

/// @fn init_page_table
/// @brief   AArch64 page table initialization.
/// @details The bootstrap code creates an initial page table to map the kernel,
///          DTB (if present), and peripherals into virtual memory using 2 MiB
///          sections. Typically:
///
///          PGD          PUD          PMD
///          000    ->    000    ->    0000 0000 0000 0000 (Kernel)
///                       040    ->    0000 0000 0800 0000 (DTB)
///                       1f8    ->    0000 0000 3f00 0000 (Peripherals)
///
///          We need to expand this table out to include a PTE table that uses
///          @a page_size entries.
pub fn init_page_table(pages_start: usize, page_size: usize) {
  let config = memory::get_memory_config();
}
