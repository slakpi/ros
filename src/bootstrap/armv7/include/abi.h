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
/// Implements the ARM calling convention exit bookkeeping. Pops LR into PC to
/// return from the subroutine.
.macro fn_exit
  mov     sp, fp
  pop     {fp, pc}
.endm

#endif
