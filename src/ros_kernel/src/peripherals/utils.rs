use super::base;

/// @fn put(val: i32, to: usize)
/// @brief Write a 32-bit integer into the specified register.
/// @param[in] val Value to write.
/// @param[in] to  Register to receive the value.
pub fn put(val: i32, to: usize) {
  let addr = base::get_peripheral_register_addr(to);
  unsafe {
    *addr = val;
  }
}

/// @fn get(from: usize) -> i32
/// @brief   Read a 32-bit integer from the specified register.
/// @param[in] from Register to read.
/// @returns The register's value.
pub fn get(from: usize) -> i32 {
  let addr = base::get_peripheral_register_addr(from);
  unsafe { *addr }
}

/// @fn delay(count: u64)
/// @brief Runs a delay loop.
/// @param[in] count Number of loop iterations.
pub fn delay(count: u64) {
  let mut c = count;
  while c > 0 {
    c -= 1;
  }
}
