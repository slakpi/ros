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
  bne     1f
  bl      mmu_make_ttbcr_value_long
  b       2f
1:
  eor     r0, r0
  cmp     r1, #2
  bne     2f
  bl      mmu_make_ttbcr_value_short

2:
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
  bne     1f
  bl      mmu_create_kernel_page_tables_long
  b       2f
1:
  cmp     r2, #2
  bne     2f
  bl      mmu_create_kernel_page_tables_short

2:
  fn_exit


/*----------------------------------------------------------------------------*/
/// Setup the translation table registers before enabling the MMU.
///
/// # Parameters
///
/// * r0 - The kernel virtual memory split.
///
/// # Description
///
/// Wraps `mmu_setup_ttbr_short` and `mmu_setup_ttbr_long`. Does nothing if r0
/// is not 2 or 3.
.global mmu_setup_ttbr
mmu_setup_ttbr:
  fn_entry

  cmp     r0, #3
  bne     1f
  bl      mmu_setup_ttbr_long
  b       2f
1:
  cmp     r0, #2
  bne     2f
  bl      mmu_setup_ttbr_short

2:
  fn_exit


/*----------------------------------------------------------------------------*/
/// Cleanup the translation table registers after enabling the MMU.
///
/// # Parameters
///
/// * r0 - The kernel virtual memory split.
///
/// # Description
///
/// Wraps `mmu_cleanup_ttbr_short` and `mmu_cleanup_ttbr_long`. Does nothing if
/// r0 is not 2 or 3.
.global mmu_cleanup_ttbr
mmu_cleanup_ttbr:
  fn_entry

  cmp     r0, #3
  bne     1f
  bl      mmu_cleanup_ttbr_long
  b       2f
1:
  cmp     r0, #2
  bne     2f
  bl      mmu_cleanup_ttbr_short

2:
  fn_exit
