/// AArch64 mode
 
/// To keep this in the first portion of the binary.
.section ".text.boot"

/// @brief Kernel entry point.
/// @param[in] x0 32-bit pointer to the DTB (primary core)
/// @param[in] x1 Zero
/// @param[in] x2 Zero
/// @param[in] x3 Zero
/// @param[in] x4 Address of this entry point
.globl _start
_start:
  mov     x5, x0            // Save the entry arugments

  mrs     x0, mpidr_el1     // Get the CPU ID
  and     x0, x0, #0xff
  cbz     x0, setup         // CPU  0   -> setup
  b       halt              // CPUs 1-3 -> halt

setup:
  ldr     x0, =__bss_start  // Clear the BSS
  ldr     x1, =__bss_size
  bl      memzero

  mov     sp, #4096         // TODO: Generalize this.

  mov     x0, x5            // Transfer to kernel stub, should not return
  bl      kernel_stub       // If kernel stub does return, fall through to the
                            // halt below.
halt:
  b       halt

/// @brief Zeros a memory block
/// @param[in] x0 Start address (Assumes 8-byte alignment)
/// @param[in] x1 Block size (Assumes 8-byte multiple)
memzero:
  cbz     x1, zero_end      // Zero bytes, nothing to do
zero_loop:
  str     xzr, [x0], #8     // Write 8 bytes from xzr, then increment by 8 bytes
	subs    x1, x1, #8        // Subtract 8 from size argument
	b.gt    zero_loop         // Keep looping
zero_end:
	ret
