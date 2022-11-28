use super::gpio;
use super::utils;
use core::str;

pub const AUX_ENABLES: usize = 0x00215004;
pub const AUX_MU_IO_REG: usize = 0x00215040;
pub const AUX_MU_IER_REG: usize = 0x00215044;
pub const _AUX_MU_IIR_REG: usize = 0x00215048;
pub const AUX_MU_LCR_REG: usize = 0x0021504C;
pub const AUX_MU_MCR_REG: usize = 0x00215050;
pub const AUX_MU_LSR_REG: usize = 0x00215054;
pub const _AUX_MU_MSR_REG: usize = 0x00215058;
pub const _AUX_MU_SCRATCH: usize = 0x0021505C;
pub const AUX_MU_CNTL_REG: usize = 0x00215060;
pub const _AUX_MU_STAT_REG: usize = 0x00215064;
pub const AUX_MU_BAUD_REG: usize = 0x00215068;

/// @fn uart_init()
/// @brief Intialize UART1.
pub fn uart_init() {
  let mut selector: i32 = utils::get(gpio::GPFSEL1);
  selector &= !(7 << 12);
  selector |= 2 << 12;
  selector &= !(7 << 15);
  selector |= 2 << 15;
  utils::put(selector, gpio::GPFSEL1);

  utils::put(0, gpio::GPPUD);
  utils::delay(150);
  utils::put(3 << 14, gpio::GPPUDCLK0);
  utils::delay(150);
  utils::put(0, gpio::GPPUDCLK0);

  utils::put(1, AUX_ENABLES);
  utils::put(0, AUX_MU_CNTL_REG);
  utils::put(0, AUX_MU_IER_REG);
  utils::put(3, AUX_MU_LCR_REG);
  utils::put(0, AUX_MU_MCR_REG);
  utils::put(270, AUX_MU_BAUD_REG);

  utils::put(3, AUX_MU_CNTL_REG);
}

/// @fn uart_recv() -> u8
/// @brief   Receive a byte from UART1. Blocks until the a arrives.
/// @returns The received byte.
pub fn _uart_recv() -> u8 {
  loop {
    let c = utils::get(AUX_MU_LSR_REG);
    if c & 0x00000001 != 0 {
      break;
    }
  }

  (utils::get(AUX_MU_IO_REG) & 0x000000ff) as u8
}

/// @fn uart_send(c: u8)
/// @brief Send a byte to UART1. Blocks until the UART is ready.
/// @param[in] c The byte to send.
pub fn uart_send(c: u8) {
  loop {
    let c = utils::get(AUX_MU_LSR_REG);
    if c & 0x00000020 != 0 {
      break;
    }
  }

  utils::put(c as i32, AUX_MU_IO_REG);
}

/// @fn uart_send_bytes(s: &[u8])
/// @brief Send an array of bytes to the mini UART.
/// @param[in] s The byte array to send.
pub fn uart_send_bytes(s: &[u8]) {
  for c in s {
    uart_send(*c);
  }
}

/// @fn uart_send_string(s: &str)
/// @brief Convenience function to send UTF-8 bytes to the mini UART.
/// @param[in] s The string to send.
pub fn uart_send_string(s: &str) {
  uart_send_bytes(s.as_bytes());
}
