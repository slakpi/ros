use core::ptr;

/// @var PERIPHERAL_BASE
/// @brief Peripheral base address for the Raspberry Pi board. The kernel is
///        single-threaded, so directly accessing the value is safe. However, it
///        should only be initialized once.
static mut PERIPHERAL_BASE: usize = 0;

/// @fn set_peripheral_base_addr
/// @brief   Sets the peripheral base address provided by the kernel stub.
/// @details Must only be called once upon kernel entry.
/// @param[in] base The peripheral base address.
pub fn set_peripheral_base_addr(base: usize) {
  unsafe {
    debug_assert!(PERIPHERAL_BASE == 0);
    PERIPHERAL_BASE = base;
  }
}

/// @fn get_peripheral_register_addr
/// @brief   Get a physical peripheral register address.
/// @param[in] reg The register address relative to the peripheral base address.
/// @returns The physical address of the register.
pub fn get_peripheral_register_addr(reg: usize) -> *mut u32 {
  unsafe { (PERIPHERAL_BASE + reg) as *mut u32 }
}

/// @fn peripheral_reg_put
/// @brief Write a 32-bit integer into the specified register.
/// @param[in] val Value to write.
/// @param[in] to  Register to receive the value.
pub fn peripheral_reg_put(val: u32, to: usize) {
  let addr = get_peripheral_register_addr(to);
  unsafe {
    ptr::write_volatile(addr, val);
  }
}

/// @fn peripheral_reg_get
/// @brief   Read a 32-bit integer from the specified register.
/// @param[in] from Register to read.
/// @returns The register's value.
pub fn peripheral_reg_get(from: usize) -> u32 {
  let addr = get_peripheral_register_addr(from);
  unsafe { ptr::read_volatile(addr) }
}

/// @fn peripheral_delay
/// @brief Runs a delay loop.
/// @param[in] count Number of loop iterations.
pub fn peripheral_delay(count: u64) {
  let mut c = count;
  while c > 0 {
    c -= 1;
  }
}
