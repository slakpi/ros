//! ARM Unwind Handlers
//!
//! These stubs seem to be necessary now starting with Rust 1.70. Just panic if
//! we get into one. In release builds, there should be no references to them
//! anyway.

#[no_mangle]
extern "C" fn __aeabi_unwind_cpp_pr0() {
  panic!();
}

#[no_mangle]
extern "C" fn __aeabi_unwind_cpp_pr1() {
  panic!();
}
