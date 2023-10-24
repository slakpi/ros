//! ARMv7a Page Table Bootstrap with Large Physical Address Extensions

#include "abi.h"

/// 2 MiB section virtual address layout:
///
///   +----+--------+--------------------+
///   | L1 |   L2   |       Offset       |
///   +----+--------+--------------------+
///   31  30       21                    0
///
/// 4 KiB page virtual address layout:
///
///   +----+--------+--------+-----------+
///   | L1 |   L2   |   L3   |  Offset   |
///   +----+--------+--------+-----------+
///   31  30       21       12           0


/*----------------------------------------------------------------------------*/
/// Create the bootstrap kernel pages using Large Physical Address Extensions.
///
/// # Parameters
///
/// * r0 - The base of the blob.
/// * r1 - The size of the DTB or 0 if the blob is not a DTB.
///
/// # Description
///
/// Maps the kernel and, as necessary, the DTB into 2 MiB sections. The kernel
/// will re-map the pages after determining the memory layout.
///
/// The mapping will use LPAE and long page table descriptors. The bootstrap
/// should have already set the TTBR0/TTBR1 split. This code only needs to know
/// the virtual base address to choose the correct L1 table entry.
.global mmu_create_kernel_pages_lpae
mmu_create_kernel_pages_lpae:
  fn_entry
  fn_exit
