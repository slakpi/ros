//! AArch64 Bit Manipulation Utilities
//!
//! http://aggregate.org/MAGIC/
//! http://graphics.stanford.edu/~seander/bithacks.html

pub use crate::support::bits::*;

/// Fast 64-bit address population count.
///
/// # Parameters
///
/// * `addr` - The address.
///
/// # Returns
///
/// The number of bits set to 1 in the address.
pub fn ones(addr: usize) -> usize {
  let mut n = addr;
  n -= (n >> 1) & 0x5555555555555555;
  n = ((n >> 2) & 0x3333333333333333) + (n & 0x3333333333333333);
  n = ((n >> 4) + n) & 0x0f0f0f0f0f0f0f0f;
  n += n >> 8;
  n += n >> 16;
  n += n >> 32;

  n & 0x7f
}

/// Fast 64-bit floor base-2 log of an address.
///
/// # Parameters
///
/// * `addr` - The address.
///
/// # Returns
///
/// floor( log2( addr ) ) when addr > 0, 0 otherwise.
pub fn floor_log2(addr: usize) -> usize {
  let mut n = addr;
  n |= n >> 1;
  n |= n >> 2;
  n |= n >> 4;
  n |= n >> 8;
  n |= n >> 16;
  n |= n >> 32;

  ones(n >> 1)
}

/// Fast 64-bit ceiling base-2 log of an address.
///
/// # Parameters
///
/// * `addr` - The address.
///
/// # Returns
///
/// ceiling( log2( addr ) ) when addr > 0, 0 otherwise.
pub fn ceil_log2(addr: usize) -> usize {
  // Essentially the same as the fast power of 2 check, except the wrapping
  // subtractions allow for `addr = 0`. If `addr` is a power of 2, `m = 0` and
  // `ceiling( log2( addr ) ) = log2( addr )`. Otherwise `m = ` and the result
  // is `floor( log2( addr ) ) + 1`.
  let mut m = addr & (addr.wrapping_sub(1));
  m |= !m.wrapping_sub(1);
  m >>= 63;

  let mut n = addr;
  n |= n >> 1;
  n |= n >> 2;
  n |= n >> 4;
  n |= n >> 8;
  n |= n >> 16;
  n |= n >> 32;

  ones(n >> 1) + m
}
