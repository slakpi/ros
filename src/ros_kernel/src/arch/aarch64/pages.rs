use crate::peripherals::memory;
use crate::support::{bits, dtb};

const PAGE_4K_MAX: usize = 8 * 1024 * 1024 * 1024;
const PAGE_4K: usize = 4 * 1024;
const SECTION_2M: usize = 2 * 1024 * 1024;

struct PageMetadata {
  phys: usize,
  virt: usize,
  reserved: bool,
  age: u32,
}

/// @fn init_page_tables
/// @brief   AArch64 page table initialization. Retrieves the system memory
///          layout and chooses the appropriate page size.
///
///          Assumes that the AArch64 bootstrap code has already setup the
///          following layout using 2 MiB sections:
///
///          L4           L3           L2
///          -------------------------------------------------------------------
///          000    ->    000    ->    000 | 0000 0000 0000 0000 (Kernel)
///                                    040 | 0000 0000 0800 0000 (DTB*)
///                                    1f8 | 0000 0000 3f00 0000 (Peripherals**)
///
///          *  The DTB may or may not be present.
///          ** The peripherals may be at a different address.
///
///          Assumes that the AArch64 bootstrap code setup the virtual addresses
///          using the physical addresses as offsets from the base virtual
///          address.
/// @param[in] virtual_base The base virtual address.
/// @param[in] blob         The DTB or ATAGs blob address.
/// @param[in] kernel_base  The kernel base address.
/// @param[in] kernel_size  The kernel size.
/// @param[in] pages_start  The base address of the page tables.
pub fn init_page_tables(
  virtual_base: usize,
  blob: usize,
  kernel_base: usize,
  kernel_size: usize,
  pages_start: usize,
) {
  let config = memory::get_memory_layout(virtual_base + blob).unwrap();
  let mut total_mem = 0usize;

  for r in config.get_ranges() {
    total_mem += r.size;
  }

  if total_mem <= PAGE_4K_MAX {
    init_page_tables_4k(
      &config,
      virtual_base,
      kernel_base,
      kernel_size,
      pages_start,
    );
  } else {
    // TODO: We're working with Raspberry Pi's. For now, 4 KiB pages are
    //       are sufficient for less than 8 GiB of memory.
    panic!("Unsupported memory layout.");
  }
}

/// @fn init_page_table_4k
/// @brief   AArch64 4 KiB page table initialization.
/// @details Initializes the page tables for 4 KiB pages. The new layout will
///          be as follows:
///
///          [ Kernel ][ / / / / / / / / / / / / / / / / / / / ][ Peripherals ]
///            2 MiB     Virtual Memory Area                      2 MiB
///            Align     4 KiB Align                              Align
///
///          The bootstrap code locates the kernel at the base virtual memory
///          address, so there is no reason to move it. The bootstrap code also
///          provides the L4 table as well as a L3 and L2 table. We'll go ahead
///          and keep those in memory along with the kernel. Additional tables
///          will be allocated as needed outside of the kernel's block.
///
///          TODO: Peripheral mapping is temporary. Once dynamic page allocation
///                is working, drivers will ask the kernel to map the pages in
///                this area that they need.
///
///                The DTB is not mapped in this new scheme for the same reason.
///                The DTB will be memory mapped by a driver after the page
///                tables are initialized.
fn init_page_tables_4k(
  config: &memory::MemoryConfig,
  virtual_base: usize,
  kernel_base: usize,
  kernel_size: usize,
  pages_start: usize,
) {
}
