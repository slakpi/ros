use core::ops;

/// @fn align_address_down(addr: T, boundary: T) -> T
/// @brief   Aligns an address with the start of the boundary.
/// @param[in] addr     The address to align.
/// @param[in] boundary The alignment boundary size.
/// @returns The new address.
pub fn align_down<T>(addr: T, boundary: T) -> T
where
  T: ops::BitAnd<Output = T> + ops::Not<Output = T> + ops::Sub<Output = T> + From<u8> + Copy,
{
  let b: T = boundary - 1u8.into();
  addr & !b
}

/// @fn align_address_up(addr: T, boundary: T) -> T
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
  let b: T = boundary - 1u8.into();
  (addr + b) & !b
}
