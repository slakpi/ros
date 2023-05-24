//! ARMv7a ABI Macros

#if !defined ABI_H
#define ABI_H

/*----------------------------------------------------------------------------*/
/// Implements the ARM calling convention entry bookkeeping.
.macro fn_entry
  push    {fp, lr}
  mov     fp, sp
.endm


/*----------------------------------------------------------------------------*/
/// Implements the ARM calling convention exit bookkeeping.
.macro fn_exit
  mov     sp, fp
  pop     {fp, lr}
.endm

#endif
