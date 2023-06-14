#[cfg(target_arch = "aarch64")]
pub mod aarch64;
#[cfg(target_arch = "arm")]
pub mod armv7;

#[cfg(any(target_arch = "aarch64", target_arch = "arm"))]
pub mod arm;

#[cfg(target_pointer_width = "64")]
pub mod common64;
#[cfg(target_pointer_width = "32")]
pub mod common32;

#[cfg(target_arch = "aarch64")]
pub use aarch64::*;
#[cfg(target_arch = "arm")]
pub use armv7::*;

#[cfg(target_pointer_width = "64")]
pub use common64::*;
#[cfg(target_pointer_width = "32")]
pub use common32::*;
