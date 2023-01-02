/// ARMv8 AArch64

#define RPI4_PERIPHERAL_BASE 0xFE000000
#define RPI3_PERIPHERAL_BASE 0x3F000000

#if (!defined RPI_VERSION) || (RPI_VERSION == 3)
#define PERIPHERAL_BASE RPI3_PERIPHERAL_BASE
#elif RPI_VERSION > 3
#define PERIPHERAL_BASE RPI4_PERIPHERAL_BASE
#else
#error "Invalid Raspberry Pi board version for AArch64."
#endif

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


///-------------------------------------------------------------------------------------------------
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
  mov     w5, w0

/*------------------------------------------------------------------------------
Check the CPU ID. Halt CPUs 1-3 and continue running on CPU 0.
------------------------------------------------------------------------------*/
  mrs     x0, mpidr_el1
  and     x0, x0, #3
  cbnz    x0, start_cpu_halt

/*------------------------------------------------------------------------------
Configure the processor exception levels. The bootloader should have dropped us
into EL2, so returning from this exception handler will jump to EL1 after
configuration.
------------------------------------------------------------------------------*/
  bl      init_kernel_el
  eret 

start_cpu_halt:
  b       start_cpu_halt


///-------------------------------------------------------------------------------------------------
/// @fn el2_entry()
/// @brief Dummy entry point for EL3 -> E2.
el2_entry:
  eret


///-------------------------------------------------------------------------------------------------
/// @fn el1_entry()
/// @brief   Entry point for EL2 -> EL1.
/// @details Runs once when the kernel drops from EL2 to EL1 during bootstrap.
///          All subsequent exception level changes will be EL0 -> EL1 or
///          EL1 -> EL0.
el1_entry:
/*------------------------------------------------------------------------------
Setup the stack pointer for the kernel.
TODO: This needs to be generalized once memory management is a thing.
------------------------------------------------------------------------------*/
  ldr     x0, =_start
  mov     sp, x0

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
	adr     x0, vectors
	msr     vbar_el1, x0

/*------------------------------------------------------------------------------
Restore the entry arguments and transfer control to the kernel stub. The stub
should not return. If it does, we will end up halting below.
------------------------------------------------------------------------------*/
  mov     w0, w5
  ldr     w1, =PERIPHERAL_BASE
  bl      kernel_stub

el1_cpu_halt:
  b       el1_cpu_halt


///-------------------------------------------------------------------------------------------------
/// @fn init_kernel_el()
/// @brief Initialize the kernel in the correct exception level.
init_kernel_el:
  mrs     x0, CurrentEL
  lsr     x0, x0, #2

  cmp     x0, #1
  beq     in_el1            // Skip EL2 initialization if already in EL1
  cmp     x0, #2
  beq     in_el2            // Skip EL3 initialization if already in EL2

in_el3:
  ldr     x0, =SCR_DEFAULT
  msr     scr_el3, x0

  ldr     x0, =SPSR_DEFAULT
  msr     spsr_el3, x0

  adr     x0, el2_entry
  msr     elr_el3, x0

in_el2:
  ldr     x0, =HCR_DEFAULT
  msr     hcr_el2, x0

  ldr     x0, =SPSR_DEFAULT
  msr     spsr_el2, x0

  adr     x0, el1_entry
  msr     elr_el2, x0

in_el1:
  ldr     x0, =SCTLR_DEFAULT_MMU_DISABLED
  msr     sctlr_el1, x0

  ret
