use crate::peripherals::mini_uart;
use core::fmt::{self, Write};

/// @var DBG_WRITE_BUFFER
/// @brief 2KiB static buffer for formatting strings. Since the kernel is
///        single-threaded, directly accessing the buffer is safe.
static mut DBG_WRITE_BUFFER: [u8; 2048] = [0; 2048];

/// @struct WriteBuffer
/// @brief  A thin wrapper around DBG_WRITE_BUFFER to track writes during string
///         formatting.
struct WriteBuffer<'buffer> {
  buf: &'buffer mut [u8],
  off: usize,
}

impl<'buffer> WriteBuffer<'buffer> {
  /// @fn new(buf: &'buffer mut [u8]) -> Self
  /// @brief Create a new wrapper around the specified buffer.
  /// @param[in] buf The buffer to wrap.
  pub fn new(buf: &'buffer mut [u8]) -> Self {
    WriteBuffer { buf: buf, off: 0 }
  }

  /// @fn as_bytes(&self) -> &[u8]
  /// @returns The buffer's byte array.
  pub fn as_bytes(&self) -> &[u8] {
    &self.buf
  }
}

impl<'buffer> fmt::Write for WriteBuffer<'buffer> {
  /// @fn write_str(&mut self, s: &str) -> fmt::Result
  /// @brief   Implements fmt::Write::write_str to write a string to the buffer.
  /// @param[in] s The string to write.
  /// @returns fmt::Error if the string will not fit in the space remaining.
  fn write_str(&mut self, s: &str) -> fmt::Result {
    let bytes = s.as_bytes();
    let dest = &mut self.buf[self.off..];

    if dest.len() < bytes.len() {
      return Err(fmt::Error);
    }

    let dest = &mut dest[..bytes.len()];
    dest.copy_from_slice(bytes);
    self.off += bytes.len();

    Ok(())
  }
}

/// @fn dbg_print(args: fmt::Arguments<'_>)
/// @brief Formats the arguments to a string and writes it to the mini UART.
/// @param[in] args The formatting arguments built by format_args!.
pub fn dbg_print(args: fmt::Arguments<'_>) {
  unsafe {
    let mut stream = WriteBuffer::new(&mut DBG_WRITE_BUFFER);
    match stream.write_fmt(args) {
      Ok(_) => mini_uart::uart_send_bytes(stream.as_bytes()),
      _ => mini_uart::uart_send_string("Error: dbg_print Failed to format string.\n"),
    }
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
