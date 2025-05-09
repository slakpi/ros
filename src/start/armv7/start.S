//! ARMv7a Start
 
#include "abi.h"

// SCTLR flags. See B4.1.130. Enable the MMU, expect exception vectors at the
// high address (0xffff_0000), enable the Access Flag, enable data caching.
#define SCTLR_MMU_ENABLE 1
#define SCTLR_C          (0b1 << 2)
#define SCTLR_V          (0b1 << 13)
#define SCTLR_AFE        (0b1 << 29)
#define SCTLR_FLAGS      (SCTLR_MMU_ENABLE | SCTLR_AFE | SCTLR_V | SCTLR_C)

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

// Check the CPU ID. Gate CPUs other than 0. MPIDR[1:0] is the CPU ID. After
// CPU0 initializes the kernel, the remainder of the CPUs will be released to
// set themselves up.
  mrc     p15, 0, r0, c0, c0, 5
  and     r0, r0, #3
  bl      gate_cpu

// Temporary stack setup before turning on the MMU.
  adr     r0, kernel_stack_start_rel
  ldr     r1, kernel_stack_start_rel
  add     sp, r0, r1
  mov     fp, sp

// Clear the BSS. The Rust Core Library provides a memset compiler intrinsic.
  adr     r0, bss_start_rel
  ldr     r1, bss_start_rel
  add     r0, r0, r1
  eor     r1, r1
  ldr     r2, =__bss_size
  bl      memset

// Halt the CPU if it does not support LPAE.
  bl      ext_has_long_descriptor_support
  bne     cpu_halt

// Check if the blob is a DTB, then create the kernel page tables. If the blob
// is a DTB, the DTB will be mapped into the bootstrap kernel page tables.
  mov     r0, r6
  bl      dtb_quick_check

// Create the bootstrap kernel page tables.
  mov     r1, r0
  mov     r0, r6
  ldr     r2, =__vmsplit
  bl      mmu_create_kernel_page_tables

// Save off physical addresses needed for the kernel configuration struct.
  adr     r3, kernel_start_rel
  ldr     r4, kernel_start_rel
  add     r3, r3, r4

  adr     r4, kernel_pages_start_rel
  ldr     r5, kernel_pages_start_rel
  add     r4, r4, r5

// Setup the MMU and enable it.
  bl      mmu_setup_ttbr

  ldr     r0, =__vmsplit
  bl      mmu_make_ttbcr_value
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
// virtual addressing.
  bx      r5
begin_virt_addressing:
  ldr     r0, =__vmsplit
  bl      mmu_cleanup_ttbr

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
//   | Virtual memory split         |
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

  ldr     r1, =__vmsplit
  str     r1, [fp, #-4]

// Perform the rest of the kernel initialization in Rustland.
  sub     r0, fp, #32
  bl      ros_kernel_init

// For now, just halt after initialization.
cpu_halt:
  b       cpu_halt


/*----------------------------------------------------------------------------*/
/// Gate the CPU. Assumes interrupts are disabled and that WFE will only release
/// the CPU in response to SEV. Simply returns if the CPU ID is 0.
///
/// # Parameters
///
/// * r0 - CPU ID
gate_cpu:
// no fn_entry required.

  cmp     r0, #0
  beq     1f                // Return if this is CPU 0.
  wfe                       // Otherwise, wait for SEV.
1:
// no fn_exit required.
  mov     pc, lr


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
