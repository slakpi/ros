//! Bit manipulation utilities.
//!
//! http://aggregate.org/MAGIC/
//! http://graphics.stanford.edu/~seander/bithacks.html

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
pub const fn align_down(addr: usize, boundary: usize) -> usize {
  addr & !(boundary - 1)
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
pub const fn align_up(addr: usize, boundary: usize) -> usize {
  let b = boundary - 1;
  (addr + b) & !b
}

/// Fast check if a number is a power of 2.
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
pub const fn is_power_of_2(n: usize) -> bool {
  (n != 0) && ((n & (n - 1)) == 0)
}

/// Fast least-significant bit mask.
///
/// # Parameters
///
/// `n` - The number to mask off.
///
/// # Returns
///
/// A mask for the least-significant bit in `n`.
pub const fn least_significant_bit(n: usize) -> usize {
  n & ((!n).wrapping_add(1))
}
