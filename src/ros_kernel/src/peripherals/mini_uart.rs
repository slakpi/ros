use super::base;
use super::gpio;
use core::str;

const AUX_ENABLES: usize = 0x00215004;
const AUX_MU_IO_REG: usize = 0x00215040;
const AUX_MU_IER_REG: usize = 0x00215044;
const _AUX_MU_IIR_REG: usize = 0x00215048;
const AUX_MU_LCR_REG: usize = 0x0021504C;
const AUX_MU_MCR_REG: usize = 0x00215050;
const AUX_MU_LSR_REG: usize = 0x00215054;
const _AUX_MU_MSR_REG: usize = 0x00215058;
const _AUX_MU_SCRATCH: usize = 0x0021505C;
const AUX_MU_CNTL_REG: usize = 0x00215060;
const _AUX_MU_STAT_REG: usize = 0x00215064;
const AUX_MU_BAUD_REG: usize = 0x00215068;

/// Enables the UART1.
///
/// # Description
///
/// Modifies GPFSEL1 to configure GPIO14 and GPIO15 to use their Alternate
/// Function 5 modes, UART1 TX and RX respectively. Disable Pull-up/-down. Then
/// enable and configure UART1.
///
/// The system frequency is 250 MHz. The baud register value of 270 translates
/// to a baud rate of 250 MHz / (8 * (270 + 1)) ~ 115200.
pub fn init() {
  base::peripheral_reg_put(0, gpio::GPPUD);
  base::peripheral_delay(gpio::GPIO_DELAY);
  base::peripheral_reg_put(3 << 14, gpio::GPPUDCLK0);
  base::peripheral_delay(gpio::GPIO_DELAY);
  base::peripheral_reg_put(0, gpio::GPPUDCLK0);

  base::peripheral_reg_put(1, AUX_ENABLES); // Enable UART1
  base::peripheral_reg_put(0, AUX_MU_CNTL_REG); // Disable TX and RX
  base::peripheral_reg_put(0, AUX_MU_IER_REG); // Disable interrupts
  base::peripheral_reg_put(3, AUX_MU_LCR_REG); // 8-bit data
  base::peripheral_reg_put(0, AUX_MU_MCR_REG); // RTS line is high
  base::peripheral_reg_put(270, AUX_MU_BAUD_REG);

  gpio::set_pin_function(gpio::GPIOPin::GPIO14, gpio::GPIOPinFunction::AltFn5);
  gpio::set_pin_function(gpio::GPIOPin::GPIO15, gpio::GPIOPinFunction::AltFn5);

  base::peripheral_reg_put(3, AUX_MU_CNTL_REG);
}

/// Receive a byte from UART1.
///
/// # Description
///
/// Blocks until a arrives.
///
/// # Returns
///
/// The received byte.
pub fn _recv() -> u8 {
  loop {
    let c = base::peripheral_reg_get(AUX_MU_LSR_REG);
    if c & 0x1 != 0 {
      break;
    }
  }

  (base::peripheral_reg_get(AUX_MU_IO_REG) & 0xff) as u8
}

/// Send a byte to UART1.
///
/// # Parameters
///
/// * `c` - The byte to send.
///
/// # Description
///
/// Serializes UART access and blocks until the UART is ready.
pub fn send(c: u8) {
  loop {
    let c = base::peripheral_reg_get(AUX_MU_LSR_REG);
    if c & 0x20 != 0 {
      break;
    }
  }

  base::peripheral_reg_put(c as u32, AUX_MU_IO_REG);
}

/// Send an array of bytes to the mini UART.
///
/// # Parameters
///
/// * `s` - The byte array to send.
///
/// # Description
///
/// Serializes UART access and blocks until the UART is ready.
pub fn send_bytes(s: &[u8]) {
  for c in s {
    send(*c);
  }
}

/// Convenience function to send UTF-8 bytes to the mini UART.
///
/// # Parameters
///
/// * `s` - The string to send.
///
/// # Description
///
/// Serializes UART access and blocks until the UART is ready.
pub fn send_string(s: &str) {
  send_bytes(s.as_bytes());
}
