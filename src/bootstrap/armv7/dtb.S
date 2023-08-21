//! ARMv7a DTB Utilities

// DTB magic identifier.
#define DTB_MAGIC 0xd00dfeed

/*----------------------------------------------------------------------------*/
/// Performs a quick check to see if the blob is a DTB.
///
/// # Parameters
///
/// * r0 - The blob address.
///
/// # Returns
///
/// The total size of the DTB or 0 if the blob is not a DTB.
.global dtb_quick_check
dtb_quick_check:
  // no fn_entry required.

  mov     r1, r0
  mov     r0, #0

  ldr     r2, [r1]
  rev     r2, r2

  ldr     r3, =DTB_MAGIC
  cmp     r2, r3
  bne     quick_dtb_check_exit

  ldr     r0, [r1, #4]
  rev     r0, r0

quick_dtb_check_exit:
  // no fn_exit required.
  mov     pc, lr
