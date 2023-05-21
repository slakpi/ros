//! Kernel Debug Utilities

/// Formats a string with provided arguments and writes the formatted string to
/// the debug device.
#[macro_export]
macro_rules! debug_print {
  () => {};
  ($($arg:tt)*) => {{
    $crate::arch::debug::debug_print(format_args!($($arg)*));
  }}
}
