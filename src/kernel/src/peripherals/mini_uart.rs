use super::base;
use super::gpio;
use super::utils;
use core::str;

pub const AUX_ENABLES: usize = 0x00215004;
pub const AUX_MU_IO_REG: usize = 0x00215040;
pub const AUX_MU_IER_REG: usize = 0x00215044;
pub const AUX_MU_IIR_REG: usize = 0x00215048;
pub const AUX_MU_LCR_REG: usize = 0x0021504C;
pub const AUX_MU_MCR_REG: usize = 0x00215050;
pub const AUX_MU_LSR_REG: usize = 0x00215054;
pub const AUX_MU_MSR_REG: usize = 0x00215058;
pub const AUX_MU_SCRATCH: usize = 0x0021505C;
pub const AUX_MU_CNTL_REG: usize = 0x00215060;
pub const AUX_MU_STAT_REG: usize = 0x00215064;
pub const AUX_MU_BAUD_REG: usize = 0x00215068;

pub fn uart_init() {
  let mut selector: i32 = utils::get(gpio::GPFSEL1);
  selector &= !(7 << 12); // clean gpio14
  selector |= 2 << 12; // set alt5 for gpio14
  selector &= !(7 << 15); // clean gpio15
  selector |= 2 << 15; // set alt5 for gpio15
  utils::put(selector, gpio::GPFSEL1);

  utils::put(0, gpio::GPPUD);
  utils::delay(150);
  utils::put(3 << 14, gpio::GPPUDCLK0);
  utils::delay(150);
  utils::put(0, gpio::GPPUDCLK0);

  utils::put(1, AUX_ENABLES); // Enable mini UART
  utils::put(0, AUX_MU_CNTL_REG); // Disable auto flow control and disable transceiver
  utils::put(0, AUX_MU_IER_REG); // Disable receive and transmit interrupts
  utils::put(3, AUX_MU_LCR_REG); // Enable 8-bit mode
  utils::put(0, AUX_MU_MCR_REG); // Set RTS line to be always high
  utils::put(270, AUX_MU_BAUD_REG); // Set baud rate to 115200

  utils::put(3, AUX_MU_CNTL_REG); // Finally, enable transceiver
}

pub fn uart_recv() -> u8 {
  loop {
    let c = utils::get(AUX_MU_LSR_REG);
    if c & 0x00000001 != 0 {
      break;
    }
  }

  (utils::get(AUX_MU_IO_REG) & 0x000000ff) as u8
}

pub fn uart_send(c: u8) {
  loop {
    let c = utils::get(AUX_MU_LSR_REG);
    if c & 0x00000020 != 0 {
      break;
    }
  }

  utils::put(c as i32, AUX_MU_IO_REG);
}

pub fn uart_send_string(s: &str) {
  for c in s.bytes() {
    uart_send(c);
  }
}
