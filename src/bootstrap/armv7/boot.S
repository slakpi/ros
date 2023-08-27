//! ARMv7a Bootstrap
 
#include "abi.h"
#include "mmu.h"

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

// Temporary stack setup before turning on the MMU.
  adr     r0, kernel_stack_start_rel
  ldr     r1, kernel_stack_start_rel
  add     sp, r0, r1
  mov     fp, sp

// Clear the BSS. The Rust Core Library provides a memset compiler intrinsic.
  adr     r0, bss_start_rel
  ldr     r1, bss_start_rel
  add     r0, r0, r1
  mov     r1, #0
  ldr     r2, =__bss_size
  bl      memset

// Check if the blob is a DTB, then create the kernel page tables. If the blob
// is a DTB, the DTB will be mapped into the bootstrap kernel pages.
  mov     r0, r6
  bl      dtb_quick_check

  mov     r1, r0
  mov     r0, r6
  bl      create_kernel_pages

// Setup the MMU. The identity map in ttbr0 is going to allow us to get to the
// next instruction after we switch on the MMU.
  adr     r0, kernel_id_pages_start_rel
  ldr     r1, kernel_id_pages_start_rel
  add     r0, r0, r1
  mcr     p15, 0, r0, c2, c0, 0

  adr     r0, kernel_pages_start_rel
  ldr     r1, kernel_pages_start_rel
  add     r0, r0, r1
  mcr     p15, 0, r0, c2, c0, 1

  ldr     r0, =TTBCR_VALUE
  mcr     p15, 0, r0, c2, c0, 2

  ldr     r0, =DACR_VALUE
  mcr     p15, 0, r0, c3, c0, 0

  ldr     r3, =begin_virt_addressing
  ldr     r4, =ros_kernel

  isb
  mrc     p15, 0, r0, c1, c0, 0
  orr     r0, r0, #MMU_ENABLE
  mcr     p15, 0, r0, c1, c0, 0
  isb

// Jump using our first virtual address to switch the program counter over to
// virtual addressing. Once the program counter is using virtual addresses,
// clear ttbr0_el1, we no longer need the temporary identity map.
  bx      r3
begin_virt_addressing:
  mov     r0, #0
  mcr     p15, 0, r0, c2, c0, 0

// Real stack setup.
  ldr     r0, =__kernel_stack_start
  mov     sp, r0
  mov     fp, sp
  sub     sp, sp, #(8 * 4)

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


kernel_stack_start_rel:
  .word __kernel_stack_start - kernel_stack_start_rel
kernel_id_pages_start_rel:
  .word __kernel_id_pages_start - kernel_id_pages_start_rel
kernel_pages_start_rel:
  .word __kernel_pages_start - kernel_pages_start_rel
bss_start_rel:
  .word __bss_start - bss_start_rel
