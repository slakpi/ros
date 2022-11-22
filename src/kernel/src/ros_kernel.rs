use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn ros_kernel() -> ! {
  loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
  loop {}
}
