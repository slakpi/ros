#[cfg(target_arch = "aarch64")]
use crate::arch::aarch64 as arch;

#[cfg(target_arch = "arm")]
use crate::arch::armv7 as arch;

/// @fn init_exception_vectors
/// @brief Architecture-independent exception vector initialization.
pub fn init_exception_vectors() {
  arch::exceptions::init_exception_vectors();
}
