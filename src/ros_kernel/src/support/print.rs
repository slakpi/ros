/// @file print.rs
/// @brief Kernel Printing Utilities
use core::fmt;

/// @var WRITE_BUFFER
/// @brief 2KiB static buffer for formatting strings. Since the kernel is
///        single-threaded, directly accessing the buffer is safe.
static mut WRITE_BUFFER: [u8; 2048] = [0; 2048];

/// @struct WriteBuffer
/// @brief  A thin wrapper around WRITE_BUFFER to track writes during string
///         formatting.
pub struct WriteBuffer<'buffer> {
  buf: &'buffer mut [u8],
  offset: usize,
}

impl<'buffer> WriteBuffer<'buffer> {
  /// @fn WriteBuffer::new
  /// @brief Create a new wrapper around the specified buffer.
  /// @param[in] buf The buffer to wrap.
  pub fn new(buf: &'buffer mut [u8]) -> Self {
    WriteBuffer {
      buf: buf,
      offset: 0,
    }
  }

  /// @fn WriteBuffer::as_bytes
  /// @returns The buffer's byte array.
  pub fn as_bytes(&self) -> &[u8] {
    &self.buf[..self.offset]
  }
}

impl<'buffer> fmt::Write for WriteBuffer<'buffer> {
  /// @fn WriteBuffer::write_str
  /// @brief   Implements fmt::Write::write_str to write a string to the buffer.
  /// @param[in] s The string to write.
  /// @returns fmt::Error if the string will not fit in the space remaining.
  fn write_str(&mut self, s: &str) -> fmt::Result {
    let bytes = s.as_bytes();
    let dest = &mut self.buf[self.offset..];

    if dest.len() < bytes.len() {
      return Err(fmt::Error);
    }

    let dest = &mut dest[..bytes.len()];
    dest.copy_from_slice(bytes);
    self.offset += bytes.len();

    Ok(())
  }
}

/// @fn new_string_format_buffer
/// @brief   Get a string format buffer.
/// @details The string format buffer wraps static memory that should only be
///          used single-threaded.
/// @returns Returns a new string format buffer.
pub fn new_string_format_buffer() -> WriteBuffer<'static> {
  unsafe { WriteBuffer::new(&mut WRITE_BUFFER) }
}
