/// ARMv8 AArch64

#include "abi.h"
#include "mmu.h"
#include "rpi_peripherals.h"

/*------------------------------------------------------------------------------
EL3 secure configuration default. Levels lower than EL3 are not secure and EL2
uses AArch64.
------------------------------------------------------------------------------*/
#define SCR_EL3_NS      (1 <<  0)
#define SCR_EL3_RW      (1 << 10)
#define SCR_EL3_DEFAULT (SCR_EL3_RW | SCR_EL3_NS)

/*------------------------------------------------------------------------------
Saved program status register defaults. Mask all interrupts. Use the EL2 stack
pointer for EL3 and the EL1 stack pointer for EL2.
------------------------------------------------------------------------------*/
#define SPSR_MASK_ALL_INTERRUPTS (7 << 6)
#define SPSR_EL3_SP              (9 << 0)
#define SPSR_EL2_SP              (5 << 0)
#define SPSR_EL3_DEFAULT         (SPSR_MASK_ALL_INTERRUPTS | SPSR_EL3_SP)
#define SPSR_EL2_DEFAULT         (SPSR_MASK_ALL_INTERRUPTS | SPSR_EL2_SP)

/*------------------------------------------------------------------------------
EL2 hypervisor configuration default. EL1 uses AArch64.
------------------------------------------------------------------------------*/
#define HCR_EL2_RW      (1 << 31)
#define HCR_EL2_DEFAULT (HCR_EL2_RW)

/*------------------------------------------------------------------------------
EL1 system control register default. Set the required reserved bits to 1 per
D17.2.118. Leave EL1 and EL0 in little endian and leave the MMU disabled.
------------------------------------------------------------------------------*/
#define SCTLR_EL1_RESERVED   ((3 << 28) | (3 << 22) | (1 << 20) | (1 << 11))
#define SCTLR_EL1_MMU_ENABLE (1 << 0)
#define SCTLR_EL1_DEFAULT    (SCTLR_EL1_RESERVED)


// The linker script forces this section to reside at the kernel base address.
.section ".text.boot"


///-------------------------------------------------------------------------------------------------
/// @fn _start
/// @brief Kernel entry point.
/// @param[in] x0 32-bit pointer to the DTB (primary core)
/// @param[in] x1 Zero
/// @param[in] x2 Zero
/// @param[in] x3 Zero
/// @param[in] x4 Address of this entry point
.global _start
_start:
/*------------------------------------------------------------------------------
Save the entry arguments
------------------------------------------------------------------------------*/
  mov     w19, w0

/*------------------------------------------------------------------------------
Check the CPU ID. Halt CPUs 1-3 and continue running on CPU 0.
------------------------------------------------------------------------------*/
  mrs     x9, mpidr_el1
  and     x9, x9, #3
  cbnz    x9, start_cpu_halt

/*------------------------------------------------------------------------------
Configure the processor exception levels. The bootloader should have dropped us
into EL2, so returning from this exception handler will jump to EL1 after
configuration.
------------------------------------------------------------------------------*/
  bl      _init_kernel_el
  eret 

start_cpu_halt:
  b       start_cpu_halt


.section ".text"


///-------------------------------------------------------------------------------------------------
/// @fn _el2_entry
/// @brief Dummy entry point for EL3 -> E2.
_el2_entry:
  eret


///-------------------------------------------------------------------------------------------------
/// @fn _el1_entry
/// @brief   Entry point for EL2 -> EL1.
/// @details Runs once when the kernel drops from EL2 to EL1 during bootstrap.
///          All subsequent exception level changes will be EL0 -> EL1 or
///          EL1 -> EL0.
_el1_entry:
/*------------------------------------------------------------------------------
Temporary stack setup before turning on the MMU.
------------------------------------------------------------------------------*/
  adrp    x9, __kernel_stack_start
  mov     sp, x9
  mov     x29, sp

/*------------------------------------------------------------------------------
Clear the BSS. The Rust Core Library provides a memset compiler intrinsic.
------------------------------------------------------------------------------*/
  adrp    x0, __bss_start
  mov     x1, #0
  ldr     x2, =__bss_size
  bl      memset

/*------------------------------------------------------------------------------
Make the blob pointer 64-bit. Check if the blob is a DTB, then create the
kernel page tables. If the blob is a DTB, the DTB will be mapped into the boot-
strap kernel pages.
------------------------------------------------------------------------------*/
  ldr     x9, =0xffffffff
  and     x19, x19, x9
  mov     x0, x19
  bl      dtb_quick_check

  mov     x1, x0            // dtb_quick_check return value to x1
  mov     x0, x19           // blobl address to x0
  bl      create_kernel_pages

/*------------------------------------------------------------------------------
Setup the MMU. The identity map in ttbr0_el1 is going to allow us to get to the
next instruction after we switch on the MMU.
------------------------------------------------------------------------------*/
  adrp    x9, __kernel_id_pages_start
  msr     ttbr0_el1, x9

  adrp    x9, __kernel_pages_start
  msr     ttbr1_el1, x9

  ldr     x9, =TCR_EL1_VALUE
  msr     tcr_el1, x9

  ldr     x9, =MAIR_EL1_VALUE
  msr     mair_el1, x0

  ldr     x20, =begin_virt_addressing
  ldr     x21, =ros_kernel

  isb
  mrs     x9, sctlr_el1
  orr     x9, x9, #SCTLR_EL1_MMU_ENABLE
  msr     sctlr_el1, x9
  isb

/*------------------------------------------------------------------------------
Jump using our first virtual address to switch the program counter over to
virtual addressing. Once the program counter is using virtual addresses, clear
ttbr0_el1, we no longer need the temporary identity map.
------------------------------------------------------------------------------*/
  br      x20
begin_virt_addressing:
  mov     x9, #0
  msr     ttbr0_el1, x9

/*------------------------------------------------------------------------------
Real stack setup.
------------------------------------------------------------------------------*/
  ldr     x9, =__kernel_stack_start
  mov     sp, x9
  mov     x29, sp
  sub     sp, sp, #(10 * 8)

/*------------------------------------------------------------------------------
Write kernel configuration struct. Provide all addresses as physical.
------------------------------------------------------------------------------*/
  ldr     x11, =__virtual_start
  ldr     x10, =__page_size
  stp     x11, x10, [x29, #-80]

  mov     x9, x19
  ldr     x10, =PERIPHERAL_BASE
  stp     x9, x10, [x29, #-64]

  ldr     x9, =PERIPHERAL_BLOCK_SIZE
  ldr     x10, =__kernel_start
  sub     x10, x10, x11
  stp     x9, x10, [x29, #-48]

  ldr     x9, =__kernel_size
  ldr     x10, =__kernel_pages_start
  sub     x10, x10, x11
  stp     x9, x10, [x29, #-32]

  ldr     x9, =__kernel_pages_size
  str     x9, [x29, #-16]

/*------------------------------------------------------------------------------
Transfer control to the kernel stub. The stub should not return. If it does, we
will end up halting below.
------------------------------------------------------------------------------*/
  sub     x0, x29, #80
  br      x21

el1_cpu_halt:
  b       el1_cpu_halt


///-------------------------------------------------------------------------------------------------
/// @fn _init_kernel_el
/// @brief Initialize the kernel in the correct exception level.
_init_kernel_el:
  mrs     x9, CurrentEL
  lsr     x9, x9, #2

  cmp     x9, #1
  beq     in_el1            // Skip EL2 initialization if already in EL1
  cmp     x9, #2
  beq     in_el2            // Skip EL3 initialization if already in EL2

in_el3:
  ldr     x9, =SCR_EL3_DEFAULT
  msr     scr_el3, x9

  ldr     x9, =SPSR_EL3_DEFAULT
  msr     spsr_el3, x9

  adr     x9, _el2_entry
  msr     elr_el3, x9

in_el2:
  ldr     x9, =HCR_EL2_DEFAULT
  msr     hcr_el2, x9

  ldr     x9, =SPSR_EL2_DEFAULT
  msr     spsr_el2, x9

  adr     x9, _el1_entry
  msr     elr_el2, x9

in_el1:
  ldr     x9, =SCTLR_EL1_DEFAULT
  msr     sctlr_el1, x9

  ret
