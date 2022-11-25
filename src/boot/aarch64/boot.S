/// ARMv8 AArch64

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
Configure the processor for Exception Level 1.
------------------------------------------------------------------------------*/
  ldr     x0, =SCTLR_DEFAULT_MMU_DISABLED
  msr     sctlr_el1, x0

  ldr     x0, =HCR_DEFAULT
  msr     hcr_el2, x0

  ldr     x0, =SCR_DEFAULT
  msr     scr_el3, x0

  ldr     x0, =SPSR_DEFAULT
  msr     spsr_el3, x0

  adr     x0, el1_entry
  msr     elr_el3, x0

el1_entry:
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
