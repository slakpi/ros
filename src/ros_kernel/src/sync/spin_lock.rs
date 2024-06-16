//! Spin Lock

use super::Sync;
use crate::arch::sync::{spin_lock, spin_unlock, try_spin_lock};
use core::ops::Drop;
use core::ptr;

/// Light-weight RAII guard object.
pub struct SpinLockGuard {
  lock_addr: usize,
}

impl SpinLockGuard {
  /// Obtain the spin lock and construct a guard object.
  ///
  /// # Parameters
  ///
  /// * `lock_addr` - The lock variable address.
  ///
  /// # Returns
  ///
  /// A guard object.
  pub fn lock(lock_addr: usize) -> Self {
    spin_lock(lock_addr);
    Self { lock_addr }
  }

  /// Try to obtain the spin lock.
  ///
  /// # Parameters
  ///
  /// * `lock_addr` - The lock variable address.
  ///
  /// # Returns
  ///
  /// A guard object if able to obtain the lock, None otherwise.
  pub fn try_lock(lock_addr: usize) -> Option<Self> {
    if !try_spin_lock(lock_addr) {
      return None;
    }

    Some(Self { lock_addr })
  }
}

impl Drop for SpinLockGuard {
  /// Release a spin lock.
  fn drop(&mut self) {
    spin_unlock(self.lock_addr);
  }
}

/// Spin lock. Holds a mutable reference to the lock variable to ensure no other
/// spin lock objects are constructed on the same lock variable.
pub struct SpinLock<'lock> {
  lock: &'lock mut u32,
}

impl<'lock> SpinLock<'lock> {
  /// Construct a new spin lock object on a lock variable.
  ///
  /// # Parameters
  ///
  /// * `lock` - The lock variable.
  ///
  /// # Returns
  ///
  /// A spin lock object.
  pub fn new(lock: &'lock mut u32) -> Self {
    SpinLock { lock }
  }
}

impl<'lock> Sync for SpinLock<'lock> {
  type Guard = SpinLockGuard;

  /// Obtain the spin lock.
  fn lock(&self) -> Self::Guard {
    Self::Guard::lock(ptr::addr_of!(self.lock) as usize)
  }

  /// Try to obtain the spin lock.
  fn try_lock(&self) -> Option<Self::Guard> {
    Self::Guard::try_lock(ptr::addr_of!(self.lock) as usize)
  }
}
