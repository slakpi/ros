/// ARMv8 AArch64

#include "abi.h"
#include "mmu.h"
#include "peripherals.h"

#define DTB_MAGIC 0xd00dfeed

#define SCTLR_RESERVED          ((3 << 28) | (3 << 22) | (1 << 20) | (1 << 11))
#define SCTLR_EE_LITTLE_ENDIAN  (1 << 25) /* EL1 endianness */
#define SCTLR_E0E_LITTLE_ENDIAN (1 << 24) /* EL0 endianness */
#define SCTLR_I_CACHE_DISABLED  (1 << 12) /* Instruction cache disabled */
#define SCTLR_D_CACHE_DISABLED  (1 <<  2) /* Data cache disabled */
#define SCTLR_MMU_ENABLED       (1 <<  0) /* MMU enabled */

/*------------------------------------------------------------------------------
Default configuration for EL1. BIG ENDIAN for EL0 and EL1, instruction and data
caches are ENABLED, MMU is DISABLED.
------------------------------------------------------------------------------*/
#define SCTLR_DEFAULT_MMU_DISABLED (SCTLR_RESERVED)

#define HCR_RW (1 << 31) /* EL1 is AArch64 */

/*------------------------------------------------------------------------------
Hypervisor configuration. EL1 runs in AArch64, EL0 runs in either AArch64 or
AArch32 depending on PSTATE.
------------------------------------------------------------------------------*/
#define HCR_DEFAULT (HCR_RW)

#define SCR_RESERVED (3 <<  4)
#define SCR_RW       (1 << 10)
#define SCR_NS       (1 <<  0)

/*------------------------------------------------------------------------------
Secure configuration default. EL2 is AArch64, EL1 controlled by EL2. Levels
lower than EL3 are not secure.
------------------------------------------------------------------------------*/
#define SCR_DEFAULT (SCR_RESERVED | SCR_RW | SCR_NS)

#define SPSR_MASK_ALL_INTERRUPTS (7 << 6)
#define SPSR_EL1h                (5 << 0) /* EL1 interrupt handler mode */

#define SPSR_DEFAULT (SPSR_MASK_ALL_INTERRUPTS | SPSR_EL1h)


// The linker script forces this section to reside at the kernel base address.
.section ".text.boot"


///-------------------------------------------------------------------------------------------------
/// @fn _start(x0, x1, x2, x3, x4)
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
/// @fn _el2_entry()
/// @brief Dummy entry point for EL3 -> E2.
_el2_entry:
  eret


///-------------------------------------------------------------------------------------------------
/// @fn _el1_entry()
/// @brief   Entry point for EL2 -> EL1.
/// @details Runs once when the kernel drops from EL2 to EL1 during bootstrap.
///          All subsequent exception level changes will be EL0 -> EL1 or
///          EL1 -> EL0.
_el1_entry:
/*------------------------------------------------------------------------------
Setup the stack and frame pointers for the kernel.
TODO: This needs to be generalized once memory management is a thing.
------------------------------------------------------------------------------*/
  ldr     x9, =__kernel_stack_start
  mov     sp, x9
  mov     x29, sp
  sub     sp, sp, #(6 * 8)

/*------------------------------------------------------------------------------
Make the blob pointer 64-bit. Check if the blob is a DTB, then create the
kernel page tables. If the blob is a DTB, the DTB will be mapped into the boot-
strap kernel pages.
------------------------------------------------------------------------------*/
  ldr     x9, =0xffffffff
  and     x19, x19, x9

  mov     x0, x19
  bl      quick_dtb_check

  mov     x1, x0
  mov     x0, x19
  bl      create_kernel_pages

/*------------------------------------------------------------------------------
Clear the BSS. The Rust Core Library provides a memset compiler intrinsic.
------------------------------------------------------------------------------*/
  ldr     x0, =__bss_start
  mov     x1, #0
  ldr     x2, =__bss_size
  bl      memset

/*------------------------------------------------------------------------------
Setup exception vectors.
------------------------------------------------------------------------------*/
  adr     x9, vectors
  msr     vbar_el1, x9

/*------------------------------------------------------------------------------
Write kernel configuration struct.
------------------------------------------------------------------------------*/
  ldr     x9, =PERIPHERAL_BASE
  str     x9, [x29, #-48]
  ldr     x9, =__page_size
  str     x9, [x29, #-40]
  ldr     x9, =__kernel_start
  str     x9, [x29, #-32]
  ldr     x9, =__kernel_size
  str     x9, [x29, #-24]
  ldr     x9, =__kernel_pages_start
  str     x9, [x29, #-16]
  ldr     x9, =__kernel_pages_size
  str     x9, [x29, #-8]

/*------------------------------------------------------------------------------
Setup the MMU.
------------------------------------------------------------------------------*/
  // Using ttbr0_el1 for now to do identity mapping.
  ldr     x9, =__kernel_pages_start
  msr     ttbr0_el1, x9

  ldr     x9, =(TCR_VALUE)
  msr     tcr_el1, x9

  ldr     x9, =(MAIR_VALUE)
  msr     mair_el1, x0

  ldr     x10, =ros_kernel
  mov     x9, #SCTLR_MMU_ENABLED
  msr     sctlr_el1, x9

/*------------------------------------------------------------------------------
Transfer control to the kernel stub. The stub should not return. If it does, we
will end up halting below.
------------------------------------------------------------------------------*/
  mov     x0, x19
  sub     x1, x29, #48
  br      x10

el1_cpu_halt:
  b       el1_cpu_halt


///-------------------------------------------------------------------------------------------------
/// @fn _init_kernel_el()
/// @brief Initialize the kernel in the correct exception level.
_init_kernel_el:
  mrs     x9, CurrentEL
  lsr     x9, x9, #2

  cmp     x9, #1
  beq     in_el1            // Skip EL2 initialization if already in EL1
  cmp     x9, #2
  beq     in_el2            // Skip EL3 initialization if already in EL2

in_el3:
  ldr     x9, =SCR_DEFAULT
  msr     scr_el3, x9

  ldr     x9, =SPSR_DEFAULT
  msr     spsr_el3, x9

  adr     x9, _el2_entry
  msr     elr_el3, x9

in_el2:
  ldr     x9, =HCR_DEFAULT
  msr     hcr_el2, x9

  ldr     x9, =SPSR_DEFAULT
  msr     spsr_el2, x9

  adr     x9, _el1_entry
  msr     elr_el2, x9

in_el1:
  ldr     x9, =SCTLR_DEFAULT_MMU_DISABLED
  msr     sctlr_el1, x9

  ret


///-------------------------------------------------------------------------------------------------
/// @fn quick_dtb_check(x0)
/// @brief   Performs a quick check to see if the blob is a DTB.
/// @param[in] x0 The blob address.
/// @returns The total size of the DTB or 0 if the blob is not a DTB.
quick_dtb_check:
  // no fn_entry required.

  mov     x9, x0
  mov     x0, #0

  ldr     w10, [x9]
  rev     w10, w10

  ldr     w11, =DTB_MAGIC
  cmp     w10, w11
  bne     quick_dtb_check_exit

  ldr     w0, [x9, #4]
  rev     w0, w0

quick_dtb_check_exit:
  // no fn_exit required.
  ret
