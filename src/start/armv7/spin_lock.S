//! ARMv7a Spin Lock

/*----------------------------------------------------------------------------*/
/// Obtain a spin lock. See D7.3.3.
///
/// # Parameters
///
/// * r0 - The lock memory location.
.global sync_spin_lock
sync_spin_lock:
// no fn_entry required.

  mov     r2, #1
1:
  ldrex   r1, [r0]          // Load the lock value.
  cmp     r1, #0            // Check if the lock value is zero.
  wfene                     // Sleep if r1 != 0.
  strexeq r1, r2, [r0]      // Attempt to write r2 if r1 was 0.
  cmpeq   r1, #0            // If r1 was 0, check if the store succeeded.
  bne     1b                // Store failed if r1 != 0, try again.
  dmb                       // Memory barrier to ensure store is observed.

// no fn_exit required.
  mov     pc, lr


/*----------------------------------------------------------------------------*/
/// Attempt to obtain a spin lock. See D7.3.3.
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

  mov     r1, r0
  mov     r2, #1
  ldrex   r0, [r1]          // Load the lock value.
  cmp     r0, #0            // Check if the lock value is zero.
  bne     1f                // Return with a non-zero value if locked.
  strex   r0, r2, [r1]      // Attempt to write r1 if r0 was 0.
  cmp     r0, #0            // Check if the store succeeded.
  bne     1f                // Return with a non-zero value if write failed.
  dmb                       // Memory barrier to ensure store is observed.

1:
// no fn_exit required.
  mov     pc, lr


/*----------------------------------------------------------------------------*/
/// Release a spin lock. See D7.3.3.
///
/// * r0 - The lock memory location.
///
/// # Description
///
///   NOTE: The caller must ensure it has obtained the lock.
.global sync_spin_unlock
sync_spin_unlock:
// no fn_entry required.

  mov     r2, #0
  dmb                       // Memory barrier to ensure stores are observed.
  str     r2, [r0]          // Clear the lock.
  dsb                       // Data synchronization barrier.
  sev                       // Send event to wake up waiting cores.

// no fn_exit required.
  mov     pc, lr
