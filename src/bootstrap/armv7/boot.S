//! ARMv7a Bootstrap
 
#include "abi.h"
#include "mmu.h"

// SCTLR flags. See B4.1.130. Enable the MMU, expect exception vectors at the
// low address (0x0000_0000), and enable the Access Flag.
//
//   TODO: Low vectors are deprecated behavior. Eventually SCTLR_V should be
//         changed to expect the vectors at the high address, 0xffff_0000, and
//         the physical vectors address should be mapped to the high address by
//         the MMU.
#define SCTLR_MMU_ENABLE 1
#define SCTLR_V          (0b0 << 13)
#define SCTLR_AFE        (0b1 << 29)
#define SCTLR_FLAGS      (SCTLR_MMU_ENABLE | SCTLR_AFE | SCTLR_V)

// DACR setup. See B4.1.43. Only using domain 0 in client mode (access
// permissions are checked).
#define DACR_VALUE 0b1

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
  bl      mmu_create_kernel_pages

// Save off physical addresses needed for the kernel configuration struct.
  adr     r3, kernel_start_rel
  ldr     r4, kernel_start_rel
  add     r3, r3, r4

  adr     r4, kernel_pages_start_rel
  ldr     r5, kernel_pages_start_rel
  add     r4, r4, r5

// Setup the MMU. The identity map in TTBR0 is going to allow us to get to the
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

  ldr     r5, =begin_virt_addressing

  isb
  mrc     p15, 0, r0, c1, c0, 0
  ldr     r1, =SCTLR_FLAGS
  orr     r0, r0, r1
  mcr     p15, 0, r0, c1, c0, 0
  isb

// Jump using our first virtual address to switch the program counter over to
// virtual addressing. Once the program counter is using virtual addresses,
// clear TTBR0, we no longer need the temporary identity map.
  bx      r5
begin_virt_addressing:
  mov     r0, #0
  mcr     p15, 0, r0, c2, c0, 0
  isb

// Real stack setup.
  ldr     r0, =__kernel_stack_start
  mov     sp, r0
  mov     fp, sp
  sub     sp, sp, #(8 * 4)

// Write kernel configuration struct. Provide all addresses as physical.
//
//   +------------------------------+ 0
//   | Virtual base address         |
//   +------------------------------+ 4
//   | Page size                    |
//   +------------------------------+ 8
//   | Physical blob address        |
//   +------------------------------+ 12
//   | Physical kernel address      |
//   +------------------------------+ 16
//   | Kernel size                  |
//   +------------------------------+ 20
//   | Physical page tables address |
//   +------------------------------+ 24
//   | Page table area size         |
//   +------------------------------+ 28
//   | / / / / / Padding / / / / /  |
//   +------------------------------+ 32
  ldr     r2, =__virtual_start
  str     r2, [fp, #-32]

  ldr     r1, =__page_size
  str     r1, [fp, #-28]

  str     r6, [fp, #-24]

  str     r3, [fp, #-20]

  ldr     r1, =__kernel_size
  str     r1, [fp, #-16]

  str     r4, [fp, #-12]

  ldr     r1, =__kernel_pages_size
  str     r1, [fp, #-8]

  mov     r1, #0
  str     r1, [fp, #-4]

// Restore the entry arguments and transfer control to the kernel stub. The stub
// should not return. If it does, we will end up halting below.
  sub     r0, fp, #32
  bl      ros_kernel

start_cpu_halt:
  b       start_cpu_halt


/*----------------------------------------------------------------------------*/
/// The ARMv7 toolchain does not support the ADRP pseudo-instruction that allows
/// getting the 4 KiB page, PC-relative address of a label within +/- 4 GiB. ADR
/// only allows getting the PC-relative address of a label within +/- 1 MiB.
///
/// We create these "relative" labels marking address that are offsets to the
/// symbols we need. We can use ADR to get the PC-relative address of the label,
/// then add the value at the label to get the PC-relative address of the actual
/// label we're interested in.
///
/// Once the MMU has been enabled, these are no longer necessary since the LDR
/// instruction can be used to get the virtual address of the label.
kernel_start_rel:
  .word __kernel_start - kernel_start_rel
kernel_stack_start_rel:
  .word __kernel_stack_start - kernel_stack_start_rel
kernel_id_pages_start_rel:
  .word __kernel_id_pages_start - kernel_id_pages_start_rel
kernel_pages_start_rel:
  .word __kernel_pages_start - kernel_pages_start_rel
bss_start_rel:
  .word __bss_start - bss_start_rel
