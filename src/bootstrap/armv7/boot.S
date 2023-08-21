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
///
/// # Description
///
/// An ARMv7a resets into the SYS operating mode. Unlike AArch64, it is not
/// necessary to set the operating mode before bootstrapping the kernel
.global _start
_start:
// Save the entry arguments.
  mov     r5, r1
  mov     r6, r2

// Check the CPU ID. Halt CPUs other than 0. MPIDR[1:0] is the CPU ID.
  mrc     p15, 0, r0, c0, c0, 5
  and     r0, r0, #3
  cmp     r0, #0
  bne     start_cpu_halt

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

// Check if the blob is a DTB, then create the kernel page tables. If the blob
// is a DTB, the DTB will be mapped into the bootstrap kernel pages.
  mov     r0, r6
  bl      dtb_quick_check

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

//TODO: Don't call into Rustland yet. Rustland assumes the MMU has been
//      bootstrapped.
// Restore the entry arguments and transfer control to the kernel stub. The stub
// should not return. If it does, we will end up halting below.
//  sub     r0, fp, #28
//  bl      ros_kernel

start_cpu_halt:
  b       start_cpu_halt
