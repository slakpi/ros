// Extern definitions for non-portable functions defined under arch/{CPU}.
extern "C" {
  fn _foo(i: isize) -> isize;
}

/// @fn foo(i: isize) -> isize
/// @brief   Unsafe wrapper for @a _foo. See See @a arch/{CPU}/foo.S.
/// @detials @a foo is never used, it is here as an infrastructure placeholder
///          until real non-portable functions are added.
#[allow(dead_code)]
pub fn foo(i: isize) -> isize {
  unsafe { _foo(i) }
}
