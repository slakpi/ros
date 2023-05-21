//! Bit manipulation utilities.
//!
//! These are slightly heavy-weight generic functions, but experiments show that
//! compiler optimizations will make these as fast as macro implementations. For
//! example, 1,000,000,000 iterations of `is_power_of_2` is as fast as a simple
//! macro implementation such as { ($n: expr) => { $n & ($n - 1) } }.
//!
//! http://aggregate.org/MAGIC/
//! http://graphics.stanford.edu/~seander/bithacks.html

use core::{cmp, ops};

/// Aligns an address with the start of the boundary.
///
/// # Parameters
///
/// * `addr` - The address to align.
/// * `boundary` - The alignment boundary.
///
/// # Assumptions
///
/// `boundary` is assumed to be greater than 0. If 0, the subtraction will
/// assert.
///
/// # Returns
///
/// The aligned address.
pub fn align_down<T>(addr: T, boundary: T) -> T
where
  T: ops::BitAnd<Output = T> + ops::Not<Output = T> + ops::Sub<Output = T> + From<u8> + Copy,
{
  addr & !(boundary - 1.into())
}

/// Aligns an address with the start of the next boundary.
///
/// # Parameters
///
/// * `addr` - The address to align.
/// * `boundary` - The alignment boundary.
///
/// # Assumptions
///
/// `boundary` is assumed to be greater than 0. If 0, the subtraction will
/// assert.
///
/// # Returns
///
/// The aligned address.
pub fn align_up<T>(addr: T, boundary: T) -> T
where
  T: ops::BitAnd<Output = T>
    + ops::Not<Output = T>
    + ops::Add<Output = T>
    + ops::Sub<Output = T>
    + From<u8>
    + Copy,
{
  let b: T = boundary - 1.into();
  (addr + b) & !b
}

/// Fast check if a value is a power of 2.
///
/// # Parameters
///
/// * `n` - The number to check.
///
/// # Returns
///
/// True if the number is a power of 2, false otherwise. The check against 0
/// ensures 0 is not reported as a power of 2 and prevents the subtraction from
/// asserting.
pub fn _is_power_of_2<T>(n: T) -> bool
where
  T: ops::BitAnd<Output = T> + ops::Sub<Output = T> + cmp::PartialEq<T> + From<u8> + Copy,
{
  (n != 0.into()) && ((n & (n - 1.into())) == 0.into())
}
