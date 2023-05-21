//! ARMv7a Bit Manipulation Utilities
//!
//! http://aggregate.org/MAGIC/
//! http://graphics.stanford.edu/~seander/bithacks.html

pub use crate::support::bits::*;

/// Fast 32-bit population count.
///
/// # Parameters
///
/// * `n` - The number.
///
/// # Returns
///
/// The number of bits set to 1 in `n`.
fn ones(n: u32) -> u32 {
  let mut n = n;
  n -= (n >> 1) & 0x55555555;
  n = ((n >> 2) & 0x33333333) + (n & 0x33333333);
  n = ((n >> 4) + n) & 0x0f0f0f0f;
  n += n >> 8;
  n += n >> 16;

  n & 0x3f
}

/// Fast 32-bit floor base-2 log.
///
/// # Parameters
///
/// * `n` - The number.
///
/// # Returns
///
/// floor( log2( n ) ) when n > 0, 0 otherwise.
pub fn floor_log2(n: u32) -> u32 {
  let mut n = n;
  n |= n >> 1;
  n |= n >> 2;
  n |= n >> 4;
  n |= n >> 8;
  n |= n >> 16;

  ones(n >> 1)
}

/// Fast 32-bit ceiling base-2 log.
///
/// # Parameters
///
/// * `n` - The number.
///
/// # Returns
///
/// ceiling( log2( n ) ) when n > 0, 0 otherwise.
fn ceil_log2(n: u32) -> u32 {
  let mut m = n & (n.wrapping_sub(1));
  m |= !m.wrapping_sub(1);
  m >>= 31;

  let mut n = n;
  n |= n >> 1;
  n |= n >> 2;
  n |= n >> 4;
  n |= n >> 8;
  n |= n >> 16;

  ones(n >> 1) + m
}
