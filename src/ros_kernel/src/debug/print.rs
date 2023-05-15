//! Kernel Debug Printing Utilities

use crate::peripherals::mini_uart;
use crate::support::print;
use core::fmt::{self, Write};

const PRINT_BUFFER_SIZE: usize = 2048;

/// Formats the arguments to a string and writes it to the mini UART.
///
/// # Parameters
///
/// * `args` - The formatting arguments built by format_args!.
pub fn dbg_print(args: fmt::Arguments) {
  let mut buf = [0u8; PRINT_BUFFER_SIZE];
  let mut stream = print::WriteBuffer::new(&mut buf);
  match stream.write_fmt(args) {
    Ok(_) => mini_uart::send_bytes(stream.as_bytes()),
    _ => mini_uart::send_string("Error: dbg_print Failed to format string.\n"),
  }
}

/// Formats a string with provided arguments and writes the formatted string to
/// the mini UART.
#[macro_export]
macro_rules! dbg_print {
  () => {};
  ($($arg:tt)*) => {{
    $crate::debug::print::dbg_print(format_args!($($arg)*));
  }}
}
