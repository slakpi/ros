extern crate cbindgen;

use std::env;
use std::path::Path;

fn run_cbindgen() {
  let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
  let bin_dir = env::var("CORROSION_BUILD_DIR").unwrap();
  let bindings_hdr = Path::new(&bin_dir).join("ros_kernel.h");

  cbindgen::generate(crate_dir)
    .expect("Failed to generate bindings.")
    .write_to_file(&bindings_hdr);
}

fn main() {
  run_cbindgen();
}
