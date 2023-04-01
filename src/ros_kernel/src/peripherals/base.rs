use core::ptr;

/// Peripheral base virtual address. The kernel is single-threaded, so directly
/// accessing the value is safe. However, it should only be initialized once.
///
///     TODO: Remove once devices are configured using the DTB.
static mut PERIPHERAL_BASE: usize = 0;

/// Sets the peripheral base virtual address provided by the DTB.
///
/// # Parameters
///
/// * `base` - The base base virtual address.
///
/// # Details
///
/// Must only be called once upon kernel entry.
///
///     TODO: Remove once devices are configured using the DTB.
pub fn set_peripheral_base_addr(base: usize) {
  unsafe {
    debug_assert!(PERIPHERAL_BASE == 0);
    PERIPHERAL_BASE = base;
  }
}

/// Get a peripheral register address.
///
/// # Parameters
///
/// * `reg` - The register address relative to the peripheral base address.
///
/// # Returns
///
/// The address of the register.
pub fn get_peripheral_register_addr(reg: usize) -> *mut u32 {
  unsafe { (PERIPHERAL_BASE + reg) as *mut u32 }
}

/// Write a 32-bit integer into the specified register.
///
/// # Parameters
///
/// `val` - Value to write.
/// `to` - Register to receive the value.
pub fn peripheral_reg_put(val: u32, to: usize) {
  let addr = get_peripheral_register_addr(to);
  unsafe {
    ptr::write_volatile(addr, val);
  }
}

/// Read a 32-bit integer from the specified register.
///
/// # Parameters
///
/// `from` - Register to read.
///
/// # Returns
///
/// The register's value.
pub fn peripheral_reg_get(from: usize) -> u32 {
  let addr = get_peripheral_register_addr(from);
  unsafe { ptr::read_volatile(addr) }
}

/// Runs a delay loop.
///
/// # Parameters
///
/// `count` - Number of loop iterations.
pub fn peripheral_delay(count: u64) {
  let mut c = count;
  while c > 0 {
    c -= 1;
  }
}
