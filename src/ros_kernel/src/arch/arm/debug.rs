//! ARM Debug Printing Utilities

use crate::peripherals::mini_uart;
use crate::support::print;
use core::fmt::{self, Write};

const PRINT_BUFFER_SIZE: usize = 2048;

/// Formats the arguments to a string and writes it to the mini UART.
///
/// # Parameters
///
/// * `args` - The formatting arguments built by format_args!.
pub fn debug_print(args: fmt::Arguments) {
  let mut buf = [0u8; PRINT_BUFFER_SIZE];
  let mut stream = print::WriteBuffer::new(&mut buf);
  match stream.write_fmt(args) {
    Ok(_) => mini_uart::send_bytes(stream.as_bytes()),
    _ => mini_uart::send_string("Error: debug_print Failed to format string.\n"),
  };
}
