#if !defined ABI_H
#define ABI_H

///-------------------------------------------------------------------------------------------------
/// @def fn_entry
/// @brief Implements the AArch32 calling convention entry bookkeeping.
.macro fn_entry
  push    {fp, lr}
  mov     fp, sp
.endm


///-------------------------------------------------------------------------------------------------
/// @def fn_exit
/// @brief Implements the AArch32 calling convention exit bookkeeping.
.macro fn_exit
  mov     sp, fp
  pop     {fp, lr}
.endm

#endif
