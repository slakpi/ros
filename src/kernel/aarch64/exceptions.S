/// @def EXCEPTION_FRAME_SIZE
/// @brief Size of the exception handler stack frame.
#define EXCEPTION_FRAME_SIZE 256


/// @def ventry(label)
/// @brief   Adds @a label as a vector to the vector table.
/// @details @a .align aligns the entries to offsets of 0x80. The GNU assembler
///          interprets @a .align as a number of bits for ARM targets.
.macro ventry label
.align 7
  b       \label
.endm


/// @def kernel_entry()
/// @brief   Exception handler prologue.
/// @details NOTE: Only the integer general purpose registers are saved. EL1 is
///          configured to trap floating-point and vector instructions to ensure
///          they are not used.
.macro kernel_entry
  sub     sp, sp, #EXCEPTION_FRAME_SIZE
  stp     x0, x1, [sp, #16 * 0]
  stp     x2, x3, [sp, #16 * 1]
  stp	    x4, x5, [sp, #16 * 2]
  stp	    x6, x7, [sp, #16 * 3]
  stp	    x8, x9, [sp, #16 * 4]
  stp	    x10, x11, [sp, #16 * 5]
  stp	    x12, x13, [sp, #16 * 6]
  stp	    x14, x15, [sp, #16 * 7]
  stp	    x16, x17, [sp, #16 * 8]
  stp	    x18, x19, [sp, #16 * 9]
  stp	    x20, x21, [sp, #16 * 10]
  stp	    x22, x23, [sp, #16 * 11]
  stp	    x24, x25, [sp, #16 * 12]
  stp	    x26, x27, [sp, #16 * 13]
  stp	    x28, x29, [sp, #16 * 14]
  str	    x30, [sp, #16 * 15] 
.endm


/// @def kernel_exit()
/// @brief Exception handler epilogue.
.macro kernel_exit
  ldp	    x0, x1, [sp, #16 * 0]
  ldp	    x2, x3, [sp, #16 * 1]
  ldp	    x4, x5, [sp, #16 * 2]
  ldp	    x6, x7, [sp, #16 * 3]
  ldp	    x8, x9, [sp, #16 * 4]
  ldp	    x10, x11, [sp, #16 * 5]
  ldp	    x12, x13, [sp, #16 * 6]
  ldp	    x14, x15, [sp, #16 * 7]
  ldp	    x16, x17, [sp, #16 * 8]
  ldp	    x18, x19, [sp, #16 * 9]
  ldp	    x20, x21, [sp, #16 * 10]
  ldp	    x22, x23, [sp, #16 * 11]
  ldp	    x24, x25, [sp, #16 * 12]
  ldp	    x26, x27, [sp, #16 * 13]
  ldp	    x28, x29, [sp, #16 * 14]
  ldr	    x30, [sp, #16 * 15] 
  add	    sp, sp, #EXCEPTION_FRAME_SIZE
  eret
.endm


/// @def vectors
/// @brief   Exception vector table.
/// @details Aligned to 0x800 (0x80 * 16).
.align 11
.globl vectors 
vectors:
  ventry  _trap_exception   // EL1t Synchronous
  ventry  _trap_exception   // EL1t IRQ
  ventry  _trap_exception   // EL1t FIQ
  ventry  _trap_exception   // EL1t Error

  ventry  _trap_exception   // EL1h Synchronous
  ventry  _trap_exception   // EL1h IRQ
  ventry  _trap_exception   // EL1h FIQ
  ventry  _trap_exception   // EL1h Error

  ventry  _trap_exception   // EL0 Synchronous (64-bit)
  ventry  _trap_exception   // EL0 IRQ (64-bit)
  ventry  _trap_exception   // EL0 FIQ (64-bit)
  ventry  _trap_exception   // EL0 Error (64-bit)

  ventry  _trap_exception   // EL0 Synchronous (32-bit)
  ventry  _trap_exception   // EL0 IRQ (32-bit)
  ventry  _trap_exception   // EL0 FIQ (32-bit)
  ventry  _trap_exception   // EL0 Error (32-bit)


/// @fn _trap_exception
/// @brief Default exception trap stub.
_trap_exception:
  kernel_entry
  mrs     x0, esr_el1
  mrs     x1, far_el1
  bl      trap_exception    // Transfer to Rustland
  // For now just halt.
  // kernel_exit
halt:
  b       halt
