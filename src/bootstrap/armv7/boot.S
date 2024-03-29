//! ARMv7a Bootstrap
 
#include "abi.h"

// The linker script forces this section to reside at the kernel base address.
.section ".text.boot"


/*----------------------------------------------------------------------------*/
/// Kernel entry point.
///
/// # Parameters
///
/// * r0 - Zero
/// * r1 - Machine ID
/// * r2 - Start of ATAGS
.global _start
_start:
// Save the entry arguments.
  mov     r5, r1
  mov     r6, r2

// Check the CPU ID. Halt CPUs other than 0. MPIDR[1:0] is the CPU ID.
  mrc     p15, 0, r0, c0, c0, 5
  and     r0, r0, #3
  cmp     r0, #0
  bne     halt

// Setup the stack pointer for the kernel.
  ldr     r0, =__kernel_stack_start
  mov     sp, r0
  mov     fp, sp
  sub     sp, sp, #(8 * 4)

// Clear the BSS. The Rust Core Library provides a memset compiler intrinsic.
  ldr     r0, =__bss_start
  mov     r1, #0
  ldr     r2, =__bss_end
  bl      memset

// Write kernel configuration struct.
  ldr     r2, =__virtual_start
  str     r2, [fp, #-28]
  ldr     r1, =__page_size
  str     r1, [fp, #-24]
  add     r1, r6, r2
  str     r1, [fp, #-20]
  ldr     r1, =__kernel_start
  str     r1, [fp, #-16]
  ldr     r1, =__kernel_size
  str     r1, [fp, #-12]
  mov     r1, #0
  str     r1, [fp, #-8]
  mov     r1, #0
  str     r1, [fp, #-4]

// Restore the entry arguments and transfer control to the kernel stub. The stub
// should not return. If it does, we will end up halting below.
  sub     r0, fp, #28
  bl      ros_kernel

halt:
  b       halt
