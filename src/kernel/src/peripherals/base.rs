static mut peripheral_base: usize = 0;

/// @fn set_peripheral_base_addr(base: usize)
/// @brief   Sets the peripherial base address provided by the kernel stub.
/// @details Must only be called once upon kernel entry.
/// @param[in] base The peripheral base address.
pub fn set_peripheral_base_addr(base: usize) {
  unsafe {
    assert!(peripheral_base == 0);
    peripheral_base = base;
  }
}

/// @fn get_peripheral_register_addr(reg: usize) -> *mut i32
/// @brief   Get a physical peripheral register address.
/// @param[in] reg The register address relative to the peripheral base address.
/// @returns The physical address of the register.
pub fn get_peripheral_register_addr(reg: usize) -> *mut i32 {
  unsafe { (peripheral_base + reg) as *mut i32 }
}
