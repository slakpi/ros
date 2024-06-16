//! Synchronization

pub mod spin_lock;

pub use spin_lock::*;

pub trait Sync {
  type Guard;

  /// Obtain the spin lock.
  ///
  /// # Returns
  ///
  /// A guard object.
  fn lock(&self) -> Self::Guard;

  /// Try to obtain the spin lock.
  ///
  /// # Returns
  ///
  /// A guard object if able to obtain the lock, None otherwise.
  fn try_lock(&self) -> Option<Self::Guard>;
}
