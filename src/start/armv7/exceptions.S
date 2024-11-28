//! ARMv7a Exception Handling

// Size of the exception handler stack frame.
#define EXCEPTION_FRAME_SIZE 64


/*----------------------------------------------------------------------------*/
/// Adds `label` as a vector to the vector table.
.macro ventry label
  ldr     pc, \label
.endm


/*----------------------------------------------------------------------------*/
/// Exception handler prologue.
///
/// # Description
///
///   NOTE: Only the integer general purpose registers are saved.
.macro kernel_entry
  sub     sp, sp, #EXCEPTION_FRAME_SIZE
  str     r0, [sp, #4 * 0]
  str     r1, [sp, #4 * 1]
  str     r2, [sp, #4 * 2]
  str     r3, [sp, #4 * 3]
  str     r4, [sp, #4 * 4]
  str     r5, [sp, #4 * 5]
  str     r6, [sp, #4 * 6]
  str     r7, [sp, #4 * 7]
  str     r8, [sp, #4 * 8]
  str     r9, [sp, #4 * 9]
  str     r10, [sp, #4 * 10]
  str     r11, [sp, #4 * 11]
  str     r12, [sp, #4 * 12]
// Skip the stack pointer.
  str     r14, [sp, #4 * 14]
  str     r15, [sp, #4 * 15] 
.endm


/*----------------------------------------------------------------------------*/
/// Exception handler epilogue.
.macro kernel_exit
  ldr     r0, [sp, #4 * 0]
  ldr     r1, [sp, #4 * 1]
  ldr     r2, [sp, #4 * 2]
  ldr     r3, [sp, #4 * 3]
  ldr     r4, [sp, #4 * 4]
  ldr     r5, [sp, #4 * 5]
  ldr     r6, [sp, #4 * 6]
  ldr     r7, [sp, #4 * 7]
  ldr     r8, [sp, #4 * 8]
  ldr     r9, [sp, #4 * 9]
  ldr     r10, [sp, #4 * 10]
  ldr     r11, [sp, #4 * 11]
  ldr     r12, [sp, #4 * 12]
// Skip the stack pointer.
  ldr     r14, [sp, #4 * 14]
  ldr     r15, [sp, #4 * 15]
  add	    sp, sp, #EXCEPTION_FRAME_SIZE
  eret
.endm


/*----------------------------------------------------------------------------*/
/// Exception vector table. This page will be mapped to the high vectors page
/// at 0xffff_0000 once the MMU is up and running.
.section ".text.vectors"
.global vectors
vectors:
  ventry  _trap_exception_addr  // Reset
  ventry  _trap_exception_addr  // Undefined instruction
  ventry  _trap_exception_addr  // Supervisor call
  ventry  _trap_exception_addr  // Prefetch abort
  ventry  _trap_exception_addr  // Data abort
  nop                           // Not used
  ventry  _trap_exception_addr  // IRQ
  ventry  _trap_exception_addr  // FIQ


_trap_exception_addr:
  .word _trap_exception


/*----------------------------------------------------------------------------*/
/// Default exception trap stub.
///
/// TODO: Handle exceptions by type.
.section ".text.stubs"
_trap_exception:
  kernel_entry
  bl      trap_exception    // Transfer to Rustland
  // For now just halt.
  // kernel_exit
1:
  b       1b
