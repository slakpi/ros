//! Spin Lock

use crate::arch::sync::{spin_lock, spin_unlock, try_spin_lock};
use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut, Drop};
use core::ptr;

pub struct SpinLockGuard<'lock, T> {
  lock: &'lock SpinLock<T>,
}

impl<'lock, T> SpinLockGuard<'lock, T> {
  pub fn new(lock: &'lock SpinLock<T>) -> Self {
    SpinLockGuard { lock }
  }
}

impl<T> Drop for SpinLockGuard<'_, T> {
  fn drop(&mut self) {
    spin_unlock(ptr::addr_of!(self.lock.lock_var) as usize);
  }
}

impl<T> Deref for SpinLockGuard<'_, T> {
  type Target = T;

  fn deref(&self) -> &T {
    unsafe { &*self.lock.obj.get() }
  }
}

impl<T> DerefMut for SpinLockGuard<'_, T> {
  fn deref_mut(&mut self) -> &mut T {
    unsafe { &mut *self.lock.obj.get() }
  }
}

pub struct SpinLock<T> {
  obj: UnsafeCell<T>,
  lock_var: u32,
}

impl<T> SpinLock<T> {
  pub const fn new(obj: T) -> Self {
    SpinLock {
      obj: UnsafeCell::new(obj),
      lock_var: 0,
    }
  }

  pub fn lock(&self) -> SpinLockGuard<'_, T> {
    spin_lock(ptr::addr_of!(self.lock_var) as usize);
    SpinLockGuard::new(self)
  }

  pub fn try_lock(&self) -> Option<SpinLockGuard<'_, T>> {
    if !try_spin_lock(ptr::addr_of!(self.lock_var) as usize) {
      return None;
    }

    Some(SpinLockGuard::new(self))
  }
}
