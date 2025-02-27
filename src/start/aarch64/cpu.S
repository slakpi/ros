//! AArch64 CPU Utilities

/*----------------------------------------------------------------------------*/
///
/// Get the current CPU ID.
///
/// # Returns
///
/// The current CPU ID.
.global cpu_get_core_id
cpu_get_core_id:
// no fn_entry required.
  mrs     x0, mpidr_el1
  and     x0, x0, #0xff

// no fn_exit required.
  ret


/*----------------------------------------------------------------------------*/
///
/// Map the current core's stack to a different virtual address.
///
/// # Parameters
///
/// * x0 - The original virtual address of the stack's start.
/// * x1 - The new virtual address of the stack's start.
///
/// # Description
///
/// The function determines the current stack offset as `offset = sp - x0`, then
/// sets the stack pointer to `sp = x1 - offset`.
///
///   NOTE: Assumes that either: a) the new virtual address points to the same
///         physical memory as the original virtual address, or b) the stack has
///         been copied to the physical memory to which the new virtual address
///         points.
.global cpu_remap_stack
cpu_remap_stack:
// no fn_entry required.
  mov     x9, sp
  sub     x9, x0, x9
  sub     sp, x1, x9

// no fn_exit required.
  ret
