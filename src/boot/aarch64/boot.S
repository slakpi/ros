/// AArch64 mode
 
/// To keep this in the first portion of the binary.
.section ".text.boot"

/// @fn _start(x0, x1, x2, x3, x4)
/// @brief Kernel entry point.
/// @param[in] x0 32-bit pointer to the DTB (primary core)
/// @param[in] x1 Zero
/// @param[in] x2 Zero
/// @param[in] x3 Zero
/// @param[in] x4 Address of this entry point
.globl _start
_start:
/*------------------------------------------------------------------------------
Save the entry arguments
------------------------------------------------------------------------------*/
  mov     x5, x0

/*------------------------------------------------------------------------------
Check the CPU ID. Halt CPUs 1-3 and continue running on CPU 0.
------------------------------------------------------------------------------*/
  mrs     x0, mpidr_el1
  and     x0, x0, #0xff
  cbnz    x0, halt

/*------------------------------------------------------------------------------
Setup the stack pointer for the kernel.
TODO: This needs to be generalized once memory management is a thing.
------------------------------------------------------------------------------*/
  mov     sp, #4096

/*------------------------------------------------------------------------------
Clear the BSS.
------------------------------------------------------------------------------*/
  ldr     x0, =__bss_start
  ldr     x1, =__bss_size
  bl      memzero

/*------------------------------------------------------------------------------
Restore the entry arguments and transfer control to the kernel stub. The stub
should not return. If it does, we will end up halting below.
------------------------------------------------------------------------------*/
  mov     x0, x5
  bl      kernel_stub

halt:
  b       halt

/// @fn memzero(x0, x1)
/// @brief Zeros a memory block
/// @param[in] x0 Start address (Assumes 8-byte alignment)
/// @param[in] x1 Block size (Assumes 8-byte multiple)
memzero:
  cbz     x1, zero_end      // Zero bytes, nothing to do
zero_loop:
  str     xzr, [x0], #8     // Write 8 zero bytes, then increment by 8 bytes
	subs    x1, x1, #8        // Subtract 8 from size argument
	bgt     zero_loop         // Keep looping if size still greater than zero
zero_end:
	ret
