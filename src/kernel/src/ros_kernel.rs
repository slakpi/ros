use crate::peripherals;
use core::panic::PanicInfo;

#[repr(C)]
pub struct ROSKernelInit {
  pub peripheral_base: usize,
}

#[no_mangle]
pub extern "C" fn ros_kernel(init: *const ROSKernelInit) -> ! {
  unsafe {
    assert!(!init.is_null());
    peripherals::base::set_peripheral_base_addr((*init).peripheral_base);
  }

  peripherals::mini_uart::uart_init();
  peripherals::mini_uart::uart_send_string("Hello, World!\n");

  loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
  loop {}
}
