//! ARMv7a Page Table Bootstrap

#include "abi.h"

/*----------------------------------------------------------------------------*/
/// Create the bootstrap kernel pages.
///
/// # Parameters
///
/// * r0 - The base of the blob.
/// * r1 - The size of the DTB or 0 if the blob is not a DTB.
///
/// # Description
///
/// Wraps `mmu_create_kernel_pages_short` and `mmu_create_kernel_pages_long`.
///
///   NOTE: Currently only calls into `mmu_create_kernel_pages_short`.
.global mmu_create_kernel_pages
mmu_create_kernel_pages:
  fn_entry

  bl      mmu_create_kernel_pages_short

  fn_exit
