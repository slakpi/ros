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
pub fn align_down_ptr(addr: usize, boundary: usize) -> usize {
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
pub fn align_up_ptr(addr: usize, boundary: usize) -> usize {
  let b = boundary - 1;
  (addr + b) & !b
}

/// Fast check if an address is a power of 2.
///
/// # Parameters
///
/// * `n` - The address to check.
///
/// # Returns
///
/// True if the address is a power of 2, false otherwise. The check against 0
/// ensures 0 is not reported as a power of 2 and prevents the subtraction from
/// asserting.
pub fn _is_power_of_2_ptr(n: usize) -> bool {
  (n != 0) && ((n & (n - 1)) == 0)
}
