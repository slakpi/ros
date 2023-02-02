/// @file bits.rs
/// @brief   Bit Manipulation Utilities
/// @details These are slightly heavy-weight generic functions, but experiments
///          show that compiler optimizations will make these as fast as macro
///          implementations. For example, 1,000,000,000 iterations of
///          @a is_power_of_2 is as fast as a simple macro implementation such
///          as { ($n: expr) => { $n & ($n - 1) } }.
use core::{cmp, ops};

/// @fn align_down
/// @brief   Aligns an address with the start of the boundary.
/// @param[in] addr     The address to align.
/// @param[in] boundary The alignment boundary size.
/// @returns The new address.
pub fn align_down<T>(addr: T, boundary: T) -> T
where
  T: ops::BitAnd<Output = T> + ops::Not<Output = T> + ops::Sub<Output = T> + From<u8> + Copy,
{
  addr & !(boundary - 1.into())
}

/// @fn align_up
/// @brief   Aligns an address with the start of the next boundary.
/// @param[in] addr     The address to align.
/// @param[in] boundary The alignment boundary size.
/// @returns The new address.
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

/// @fn is_power_of_2
/// @brief   Fast check if a value is a power of 2.
/// @param[in] n The value to check.
/// @returns True if the value is a power of 2.
pub fn is_power_of_2<T>(n: T) -> bool
where
  T: ops::BitAnd<Output = T> + ops::Sub<Output = T> + cmp::PartialEq<T> + From<u8> + Copy,
{
  (n & (n - 1.into())) == 0.into()
}
