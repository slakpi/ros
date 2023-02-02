use core::panic::PanicInfo;

/// @fn panic
/// @brief   Panic handler.
/// @param[in] info The panic info.
/// @returns Does not return.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
  loop {}
}
