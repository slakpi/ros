use crate::peripherals::mini_uart;
use crate::support::print;
use core::fmt::{self, Write};

/// @fn dbg_print
/// @brief Formats the arguments to a string and writes it to the mini UART.
/// @param[in] args The formatting arguments built by format_args!.
pub fn dbg_print(args: fmt::Arguments<'_>) {
  let mut stream = print::new_string_format_buffer();
  match stream.write_fmt(args) {
    Ok(_) => mini_uart::send_bytes(stream.as_bytes()),
    _ => mini_uart::send_string("Error: dbg_print Failed to format string.\n"),
  }
}

/// @def dbg_print!
/// @brief Macro form that takes a format string and arguments to print to the
///        mini UART.
#[macro_export]
macro_rules! dbg_print {
  () => {};
  ($($arg:tt)*) => {{
    $crate::debug::print::dbg_print(format_args!($($arg)*));
  }}
}
