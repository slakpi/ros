//! ARMv7a Page Table Bootstrap

#include "abi.h"

/*----------------------------------------------------------------------------*/
/// Setup the TTBCR flags for the MMU.
///
/// # Parameters
///
/// * r0 - The kernel virtual memory split.
///
/// # Returns
///
/// A TTBCR that enables the appropriate table registers and extended addressing
/// support. If r0 is not 2 or 3, returns 0.
.global mmu_make_ttbcr_value
mmu_make_ttbcr_value:
  fn_entry

  mov     r1, r0
  cmp     r1, #3
  bne     ttbcr_not_3to1
  bl      mmu_make_ttbcr_value_long
  b       ttbcr_exit
ttbcr_not_3to1:
  mov     r0, #0
  cmp     r1, #2
  bne     ttbcr_exit
  bl      mmu_make_ttbcr_value_short

ttbcr_exit:
  fn_exit

/*----------------------------------------------------------------------------*/
/// Create the bootstrap kernel page tables.
///
/// # Parameters
///
/// * r0 - The base of the blob.
/// * r1 - The size of the DTB or 0 if the blob is not a DTB.
/// * r2 - The kernel virtual memory split.
///
/// # Description
///
/// Wraps `mmu_create_kernel_page_tables_short` and
/// `mmu_create_kernel_page_tables_long`. Does nothing if r2 is not 2 or 3.
.global mmu_create_kernel_page_tables
mmu_create_kernel_page_tables:
  fn_entry

  cmp     r2, #3
  bne     pages_not_3to1
  bl      mmu_create_kernel_page_tables_long
  b       pages_exit
pages_not_3to1:
  cmp     r2, #2
  bne     pages_exit
  bl      mmu_create_kernel_page_tables_short

pages_exit:
  fn_exit
