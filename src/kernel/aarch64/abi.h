#if !defined ABI_H
#define ABI_H

///-------------------------------------------------------------------------------------------------
/// @def fn_entry
/// @brief Implements the AArch64 calling convention entry bookkeeping.
.macro fn_entry
  stp     x29, x30, [sp, #-16]!
  mov     x29, sp
.endm


///-------------------------------------------------------------------------------------------------
/// @def fn_exit
/// @brief Implements the AArch64 calling convention exit bookkeeping.
.macro fn_exit
  mov     sp, x29
  ldp     x29, x30, [sp], #16
.endm

#endif
