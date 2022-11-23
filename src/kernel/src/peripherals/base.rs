static mut PERIPHERAL_BASE: usize = 0;

/// @fn set_peripheral_base_addr(base: usize)
/// @brief   Sets the peripherial base address provided by the kernel stub.
/// @details Must only be called once upon kernel entry.
/// @param[in] base The peripheral base address.
pub fn set_peripheral_base_addr(base: usize) {
  unsafe {
    assert!(PERIPHERAL_BASE == 0);
    PERIPHERAL_BASE = base;
  }
}

/// @fn get_peripheral_register_addr(reg: usize) -> *mut i32
/// @brief   Get a physical peripheral register address.
/// @param[in] reg The register address relative to the peripheral base address.
/// @returns The physical address of the register.
pub fn get_peripheral_register_addr(reg: usize) -> *mut i32 {
  unsafe { (PERIPHERAL_BASE + reg) as *mut i32 }
}
