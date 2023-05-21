//! AArch64 Bit Manipulation Utilities
//!
//! http://aggregate.org/MAGIC/
//! http://graphics.stanford.edu/~seander/bithacks.html

pub use crate::support::bits::*;

/// Fast 64-bit population count.
///
/// # Parameters
///
/// * `n` - The number.
///
/// # Returns
///
/// The number of bits set to 1 in `n`.
pub fn ones(n: u64) -> u64 {
  let mut n = n;
  n -= (n >> 1) & 0x5555555555555555;
  n = ((n >> 2) & 0x3333333333333333) + (n & 0x3333333333333333);
  n = ((n >> 4) + n) & 0x0f0f0f0f0f0f0f0f;
  n += n >> 8;
  n += n >> 16;
  n += n >> 32;

  n & 0x7f
}

/// Fast 64-bit floor base-2 log.
///
/// # Parameters
///
/// * `n` - The number.
///
/// # Returns
///
/// floor( log2( n ) ) when n > 0, 0 otherwise.
pub fn floor_log2(n: u64) -> u64 {
  let mut n = n;
  n |= n >> 1;
  n |= n >> 2;
  n |= n >> 4;
  n |= n >> 8;
  n |= n >> 16;
  n |= n >> 32;

  ones(n >> 1)
}

/// Fast 64-bit ceiling base-2 log.
///
/// # Parameters
///
/// * `n` - The number.
///
/// # Returns
///
/// ceiling( log2( n ) ) when n > 0, 0 otherwise.
pub fn ceil_log2(n: u64) -> u64 {
  let mut m = n & (n.wrapping_sub(1));
  m |= !m.wrapping_sub(1);
  m >>= 63;

  let mut n = n;
  n |= n >> 1;
  n |= n >> 2;
  n |= n >> 4;
  n |= n >> 8;
  n |= n >> 16;
  n |= n >> 32;

  ones(n >> 1) + m
}
