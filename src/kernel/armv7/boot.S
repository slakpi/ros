// ARMv7
 
#if RPI_VERSION > 2
#error "Invalid Raspberry Pi board version for ARMv7."
#endif

#define PERIPHERAL_BASE 0x3F000000


// The linker script forces this section to reside at the kernel base address.
.section ".text.boot"


///-------------------------------------------------------------------------------------------------
/// @fn _start(r0, r1, r2)
/// @brief Kernel entry point.
/// @param[in] r0 Zero
/// @param[in] r1 Machine ID
/// @param[in] r2 Start of ATAGS
.globl _start
_start:
/*------------------------------------------------------------------------------
Save the entry arguments
------------------------------------------------------------------------------*/
  mov     r5, r1
  mov     r6, r2

/*------------------------------------------------------------------------------
Check the CPU ID. Halt CPUs 1-3 and continue running on CPU 0.
------------------------------------------------------------------------------*/
  mrc     p15, 0, r0, c0, c0, 5
  and     r0, r0, #3
  cmp     r0, #0
  bne     halt

/*------------------------------------------------------------------------------
Setup the stack pointer for the kernel.
TODO: This needs to be generalized once memory management is a thing.
------------------------------------------------------------------------------*/
  ldr     r0, =_start
  mov     sp, r0

/*------------------------------------------------------------------------------
Clear the BSS. The Rust Core Library provides a memset compiler intrinsic.
------------------------------------------------------------------------------*/
  ldr     r0, =__bss_start
  mov     r1, #0
  ldr     r2, =__bss_end
  bl      memset

/*------------------------------------------------------------------------------
Restore the entry arguments and transfer control to the kernel stub. The stub
should not return. If it does, we will end up halting below.
------------------------------------------------------------------------------*/
  mov     r0, r6
  ldr     r1, =PERIPHERAL_BASE
  ldr     r2, =__page_size
  bl      ros_kernel

halt:
  b       halt
