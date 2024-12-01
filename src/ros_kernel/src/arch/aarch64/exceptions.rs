//! AArch64 Exception Management

use crate::debug_print;

/// The CPU context upon exception entry. See `exceptions.S` regarding floating-
/// point registers.
#[repr(C)]
struct CpuContext {
  x0: usize,
  x1: usize,
  x2: usize,
  x3: usize,
  x4: usize,
  x5: usize,
  x6: usize,
  x7: usize,
  x8: usize,
  x9: usize,
  x10: usize,
  x11: usize,
  x12: usize,
  x13: usize,
  x14: usize,
  x15: usize,
  x16: usize,
  x17: usize,
  x18: usize,

  x19: usize,
  x20: usize,
  x21: usize,
  x22: usize,
  x23: usize,
  x24: usize,
  x25: usize,
  x26: usize,
  x27: usize,
  x28: usize,
  x29: usize,
  x30: usize,
  sp: usize,
  
  elr_el1: usize,
  spsr_el1: usize,
}

/// AArch64 exception trap.
///
/// # Parameters
///
/// * `esr_el1` - Exception Syndrome Register value.
/// * `far_el1` - Fault Address Register value.
/// * `cpu_context` - Pointer to the saved CPU context structure.
#[no_mangle]
extern "C" fn trap_exception(esr_el1: usize, far_el1: usize, cpu_context: usize) {
  assert!(cpu_context != 0);
  
  _ = unsafe { &*(cpu_context as *const CpuContext) };

  debug_print!(
    "Fell into a trap! esr_el1={:#x}, far_el1={:#x}\n",
    esr_el1,
    far_el1,
  );
}
