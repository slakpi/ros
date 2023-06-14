//! 64-bit Bit Manipulation Utilities
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
/// The number of bits set to 1 in the number.
pub const fn ones(n: usize) -> usize {
  let mut n = n;
  n -= (n >> 1) & 0x5555555555555555;
  n = ((n >> 2) & 0x3333333333333333) + (n & 0x3333333333333333);
  n = ((n >> 4) + n) & 0x0f0f0f0f0f0f0f0f;
  n += n >> 8;
  n += n >> 16;
  n += n >> 32;

  n & 0x7f
}

/// Fast 64-bit floor base-2 log of a number.
///
/// # Parameters
///
/// * `n` - The number.
///
/// # Returns
///
/// floor( log2( n ) ) when n > 0, 0 otherwise.
pub const fn floor_log2(n: usize) -> usize {
  let mut n = n;
  n |= n >> 1;
  n |= n >> 2;
  n |= n >> 4;
  n |= n >> 8;
  n |= n >> 16;
  n |= n >> 32;

  ones(n >> 1)
}

/// Fast 64-bit ceiling base-2 log of a number.
///
/// # Parameters
///
/// * `n` - The number.
///
/// # Returns
///
/// ceiling( log2( n ) ) when n > 0, 0 otherwise.
pub const fn ceil_log2(n: usize) -> usize {
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

/// Removes all even (1-based) bits and leaves the odd bits in the lower
/// 32-bits.
///
/// # Parameters
///
/// * `n` - The number.
///
/// # Description
///
/// Given a 64-bit word 0xeoeoeoeoeoeoeoeo, where `e` is a 1-based even bit and
/// `o` is an odd bit, the function returns 0x00000000oooooooo. Each odd bit
/// maintains its relative order with the other bits.
///
/// # Returns
///
/// The odd bits moved to the lower 32-bits.
pub const fn compact_odd_bits(n: usize) -> usize {
  let mut n = n;
  n = ((n & 0x4444444444444444) >> 1)  | (n & 0x1111111111111111);
  n = ((n & 0x3030303030303030) >> 2)  | (n & 0x0303030303030303);
  n = ((n & 0x0f000f000f000f00) >> 4)  | (n & 0x000f000f000f000f);
  n = ((n & 0x00ff000000ff0000) >> 8)  | (n & 0x000000ff000000ff);
  n = ((n & 0x0000ffff00000000) >> 16) | (n & 0x000000000000ffff);
  n
}

/// Removes all odd (1-based) bits and leaves the even bits in the lower
/// 32-bits.
///
/// # Parameters
///
/// * `n` - The number.
///
/// # Description
///
/// See `compact_odd_bits`.
///
/// # Returns
///
/// The even bits moved to the lower 32-bits.
pub const fn compact_even_bits(n: usize) -> usize {
  compact_odd_bits(n >> 1)
}
