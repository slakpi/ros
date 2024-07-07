//! AArch64 Start

#include "abi.h"
#include "mmu.h"

// EL3 secure configuration default. Levels lower than EL3 are not secure and
// EL2 uses AArch64.
#define SCR_EL3_NS      (1 <<  0)
#define SCR_EL3_RW      (1 << 10)
#define SCR_EL3_DEFAULT (SCR_EL3_RW | SCR_EL3_NS)

// Saved program status register defaults. Mask all interrupts. Use the EL2
// stack pointer for EL3 and the EL1 stack pointer for EL2.
#define SPSR_MASK_ALL_INTERRUPTS (7 << 6)
#define SPSR_EL3_SP              (9 << 0)
#define SPSR_EL2_SP              (5 << 0)
#define SPSR_EL3_DEFAULT         (SPSR_MASK_ALL_INTERRUPTS | SPSR_EL3_SP)
#define SPSR_EL2_DEFAULT         (SPSR_MASK_ALL_INTERRUPTS | SPSR_EL2_SP)

// EL2 hypervisor configuration default. EL1 uses AArch64.
#define HCR_EL2_RW      (1 << 31)
#define HCR_EL2_DEFAULT (HCR_EL2_RW)

// EL1 system control register default. Set the required reserved bits to 1 per
// D17.2.118. Leave EL1 and EL0 in little endian and leave the MMU disabled.
#define SCTLR_EL1_C          (1 << 2)
#define SCTLR_EL1_RESERVED   ((3 << 28) | (3 << 22) | (1 << 20) | (1 << 11) | (3 << 7))
#define SCTLR_EL1_MMU_ENABLE (1 << 0)
#define SCTLR_EL1_DEFAULT    (SCTLR_EL1_RESERVED | SCTLR_EL1_C)


// The linker script forces this section to reside at the kernel base address.
.section ".text.boot"


/*----------------------------------------------------------------------------*/
/// Kernel entry point.
///
/// # Parameters
///
/// * w0 - 32-bit pointer to the DTB (primary core)
/// * x1 - Zero
/// * x2 - Zero
/// * x3 - Zero
/// * x4 - Address of this entry point
///
/// # Returns
///
/// Does not return.
.global _start
_start:
// Save the entry arguments
  mov     w19, w0

// Configure the processor exception levels. The bootloader should have dropped
// us into EL2, so returning from this exception handler will jump to EL1 after
// configuration.
  bl      init_kernel_el
  eret 


.section ".text"


/*----------------------------------------------------------------------------*/
/// Initialize the kernel in the correct exception level.
init_kernel_el:
  mrs     x9, CurrentEL
  lsr     x9, x9, #2

  cmp     x9, #1
  beq     1f                // Skip EL2 initialization if already in EL1
  cmp     x9, #2
  beq     2f                // Skip EL3 initialization if already in EL2

3:
  ldr     x9, =SCR_EL3_DEFAULT
  msr     scr_el3, x9

  ldr     x9, =SPSR_EL3_DEFAULT
  msr     spsr_el3, x9

  adr     x9, el2_entry
  msr     elr_el3, x9

2:
  ldr     x9, =HCR_EL2_DEFAULT
  msr     hcr_el2, x9

  ldr     x9, =SPSR_EL2_DEFAULT
  msr     spsr_el2, x9

  adr     x9, el1_entry
  msr     elr_el2, x9

1:
  ldr     x9, =SCTLR_EL1_DEFAULT
  msr     sctlr_el1, x9

  ret


/*----------------------------------------------------------------------------*/
/// Dummy entry point for EL3 -> E2.
el2_entry:
  eret


/*----------------------------------------------------------------------------*/
/// Entry point for EL2 -> EL1.
el1_entry:
  mrs     x9, mpidr_el1
  and     x9, x9, #0xff
  cbz     x9, primary_cpu_boot
  b       secondary_cpu_boot


/*----------------------------------------------------------------------------*/
/// Boot the primary CPU. While the secondary CPUs are parked and interrupts are
/// disabled, the primary CPU will perform all of the low-level kernel setup
/// that needs to be done single-threaded. When setup is complete, the primary
/// CPU will release the secondary CPUs to do their initialization.
primary_cpu_boot:
// no fn_entry required.

// Temporary stack setup before turning on the MMU.
  adrp    x9, __kernel_stack_start
  mov     sp, x9
  mov     x29, sp

// Clear the BSS. The Rust Core Library provides a memset compiler intrinsic.
  adrp    x0, __bss_start
  mov     x1, #0
  ldr     x2, =__bss_size
  bl      memset

// Make the blob pointer 64-bit. Check if the blob is a DTB, then create the
// kernel page tables. If the blob is a DTB, the DTB will be mapped into the
// initial kernel page tables.
  ldr     x9, =0xffffffff
  and     x19, x19, x9
  mov     x0, x19
  bl      dtb_quick_check

  mov     x1, x0            // dtb_quick_check return value to x1
  mov     x0, x19           // blob address to x0
  bl      mmu_create_kernel_page_tables

// Save off physical addresses needed for the kernel configuration struct.
  adrp    x21, __kernel_start
  adrp    x22, __kernel_pages_start

// Setup the MMU. The identity map in TTBR0_EL1 is going to allow us to get to
// the next instruction after we switch on the MMU.
//
//   TODO: For some reason, functionalizing this causes downstream issues when
//         the kernel runs. Attempted to do:
//
//             ldr     x0, =primary_cpu_begin_virt_addressing
//             b       setup_and_enable_mmu
//           primary_cpu_begin_virt_addressing:
//             <cleanup code>
//
//         Then:
//
//           setup_and_enable_mmu:
//             <setup code>
//             br      x0
//
//         This caused an infinite loop waiting for the UART to become available
//         on hardware, but worked fine in QEMU.
  adrp    x9, __kernel_id_pages_start
  msr     ttbr0_el1, x9

  adrp    x9, __kernel_pages_start
  msr     ttbr1_el1, x9

  ldr     x9, =TCR_EL1_VALUE
  msr     tcr_el1, x9

  ldr     x9, =MAIR_EL1_VALUE
  msr     mair_el1, x0

  ldr     x20, =primary_cpu_begin_virt_addressing

  isb
  mrs     x9, sctlr_el1
  orr     x9, x9, #SCTLR_EL1_MMU_ENABLE
  msr     sctlr_el1, x9
  isb

// Jump using our first virtual address to switch the program counter over to
// virtual addressing. Once the program counter is using virtual addresses,
// clear TTBR0_EL1, we no longer need the temporary identity map.
  br      x20
primary_cpu_begin_virt_addressing:
  mov     x9, #0
  msr     ttbr0_el1, x9
  isb

// Setup the exception vectors.
  adr     x9, vectors
  msr     vbar_el1, x9
 
// Temporary stack setup with virtual addressing enabled.
  ldr     x9, =__kernel_stack_start
  mov     sp, x9
  mov     x29, sp

// Write kernel configuration struct. Provide all addresses as physical.
//
//   +------------------------------+ 0
//   | Virtual base address         |
//   +------------------------------+ 8
//   | Page size                    |
//   +------------------------------+ 16
//   | Physical blob address        |
//   +------------------------------+ 24
//   | Physical kernel address      |
//   +------------------------------+ 32
//   | Kernel size                  |
//   +------------------------------+ 40
//   | Physical page tables address |
//   +------------------------------+ 48
//   | Page table area size         |
//   +------------------------------+ 56
//   | / / / / / Padding / / / / /  |
//   +------------------------------+ 64
  sub     sp, sp, #(8 * 8)

  ldr     x11, =__virtual_start
  ldr     x10, =__page_size
  stp     x11, x10, [x29, #-64]

  stp     x19, x21, [x29, #-48]

  ldr     x9, =__kernel_size
  stp     x9, x22, [x29, #-32]

  ldr     x9, =__kernel_pages_size
  mov     x10, #0
  stp     x9, x10, [x29, #-16]

// Perform the rest of the kernel initialization in Rustland.
  sub     x0, x29, #64
  bl      ros_kernel_init

// Jump to the scheduler.
  bl      ros_kernel_scheduler

// no fn_exit required. We are not coming back from the scheduler.


secondary_cpu_boot:
// no fn_entry required.

// Setup the MMU. The identity map in TTBR0_EL1 is going to allow us to get to
// the next instruction after we switch on the MMU.
  adrp    x9, __kernel_id_pages_start
  msr     ttbr0_el1, x9

  adrp    x9, __kernel_pages_start
  msr     ttbr1_el1, x9

  ldr     x9, =TCR_EL1_VALUE
  msr     tcr_el1, x9

  ldr     x9, =MAIR_EL1_VALUE
  msr     mair_el1, x0

  ldr     x20, =secondary_cpu_begin_virt_addressing

  isb
  mrs     x9, sctlr_el1
  orr     x9, x9, #SCTLR_EL1_MMU_ENABLE
  msr     sctlr_el1, x9
  isb

// Jump using our first virtual address to switch the program counter over to
// virtual addressing. Once the program counter is using virtual addresses,
// clear TTBR0_EL1, we no longer need the temporary identity map.
  br      x20
secondary_cpu_begin_virt_addressing:
  mov     x9, #0
  msr     ttbr0_el1, x9
  isb

// Setup the exception vectors.
  adr     x9, vectors
  msr     vbar_el1, x9
 
// Jump to the scheduler.
  bl      ros_kernel_scheduler

// no fn_exit required. We are not coming back from the scheduler.
