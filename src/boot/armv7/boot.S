// ARMv7
 
// To keep this in the first portion of the binary.
.section ".text.boot"

/// @fn _start(r0, r1, r2)
/// @brief Kernel entry point.
/// @param[in] r0 Zero
/// @param[in] r1 Machine ID
/// @param[in] r2 Start of ATAGS
.globl _start
_start:
/*------------------------------------------------------------------------------
Save the entry arguments
------------------------------------------------------------------------------*/
  mov     r5, r0
  mov     r6, r1
  mov     r7, r2

/*------------------------------------------------------------------------------
Check the CPU ID. Halt CPUs 1-3 and continue running on CPU 0.
------------------------------------------------------------------------------*/
  mrc     p15, 0, r0, c0, c0, 5
  and     r0, r0, #3
  cmp     r0, #0
  bne     halt

/*------------------------------------------------------------------------------
Setup the stack pointer for the kernel.
TODO: This needs to be generalized once memory management is a thing.
------------------------------------------------------------------------------*/
  mov     sp, #4096

/*------------------------------------------------------------------------------
Clear the BSS.
------------------------------------------------------------------------------*/
  ldr     r0, =__bss_start
  ldr     r1, =__bss_end
  bl      memzero

/*------------------------------------------------------------------------------
Restore the entry arguments and transfer control to the kernel stub. The stub
should not return. If it does, we will end up halting below.
------------------------------------------------------------------------------*/
  mov     r0, r5
  mov     r1, r6
  mov     r2, r7
  ldr     r5, =kernel_stub
  blx     r5

halt:
  b       halt

/// @fn memzero(r0, r1)
/// @brief Zeros a memory block
/// @param[in] r0 Start address (Assumes 4-byte alignment)
/// @param[in] r1 Block size (Assumes 4-byte multiple)
memzero:
  cmp     r1, #0
  beq     zero_end          // Zero bytes, nothing to do
  mov     r2, #0
zero_loop:
  str     r2, [r0], #4      // Write 4 zero bytes, then increment by 4 bytes
	subs    r1, r1, #4        // Subtract 4 from size argument
  bgt     zero_loop         // Keep looping if size still greater than zero
zero_end:
	bx      lr                // Return
