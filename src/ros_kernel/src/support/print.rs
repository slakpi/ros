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
  offs: usize,
}

impl<'buffer> WriteBuffer<'buffer> {
  /// @fn new(buf: &'buffer mut [u8]) -> Self
  /// @brief Create a new wrapper around the specified buffer.
  /// @param[in] buf The buffer to wrap.
  pub fn new(buf: &'buffer mut [u8]) -> Self {
    WriteBuffer { buf: buf, offs: 0 }
  }

  /// @fn as_bytes(&self) -> &[u8]
  /// @returns The buffer's byte array.
  pub fn as_bytes(&self) -> &[u8] {
    &self.buf[..self.offs]
  }
}

impl<'buffer> fmt::Write for WriteBuffer<'buffer> {
  /// @fn write_str(&mut self, s: &str) -> fmt::Result
  /// @brief   Implements fmt::Write::write_str to write a string to the buffer.
  /// @param[in] s The string to write.
  /// @returns fmt::Error if the string will not fit in the space remaining.
  fn write_str(&mut self, s: &str) -> fmt::Result {
    let bytes = s.as_bytes();
    let dest = &mut self.buf[self.offs..];

    if dest.len() < bytes.len() {
      return Err(fmt::Error);
    }

    let dest = &mut dest[..bytes.len()];
    dest.copy_from_slice(bytes);
    self.offs += bytes.len();

    Ok(())
  }
}

pub fn new_string_format_buffer() -> WriteBuffer<'static> {
  unsafe {
    WriteBuffer::new(&mut WRITE_BUFFER)
  }
}
