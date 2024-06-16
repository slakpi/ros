//! ARMv8 Spin Lock

/*----------------------------------------------------------------------------*/
/// Obtain a spin lock. See K13.3.4.
///
/// # Parameters
///
/// * x0 - The lock memory location.
.global sync_spin_lock
sync_spin_lock:
// no fn_entry required.

  mov     w10, #1
  sevl                      // Invalidate the first wait.
  prfm    pstl1keep, [x0]   // Allocate lock value into cache in a unique state.
1:
  wfe                       // Sleep
  ldaxr   w9, [x0]          // Load the lock value.
  cbnz    w9, 1b            // If w9 != 0, try again.
  stxr    w9, w10, [x0]     // Attempt to write w10.
  cbnz    w9, 1b            // If w9 != 0, try again.

// no fn_exit required.
  ret


/*----------------------------------------------------------------------------*/
/// Attempt to obtain a spin lock. See K13.6.2.
///
/// # Parameters
///
/// * r0 - The lock memory location.
///
/// # Returns
///
/// 0 if able to obtain the spin, non-zero otherwise.
.global sync_try_spin_lock
sync_try_spin_lock:
// no fn_entry required.

  mov     x9, x0
  mov     w10, #1
  prfm    pstl1keep, [x0]   // Allocate lock value into cache in a unique state.
  ldaxr   w0, [x9]          // Load the lock value.
  cbnz    w0, 1f            // If w0 != 0, return with a non-zero value.
  stxr    w0, w10, [x9]     // Attempt to write w10, w0 is 0 if successful.

1:
// no fn_exit required.
  ret


/*----------------------------------------------------------------------------*/
/// Release a spin lock. See K13.6.2.
///
/// # Parameters
///
/// * x0 - The lock memory location.
///
/// # Description
///
///   NOTE: The caller must ensure it has obtained the lock.
.global sync_spin_unlock
sync_spin_unlock:
// no fn_entry required.

  stlr    wzr, [x0]         // Release the lock.

// no fn_exit required.
  ret
