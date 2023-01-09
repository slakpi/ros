use core::ptr;

/// https://devicetree-specification.readthedocs.io/en/stable/flattened-format.html

pub fn check_dtb(dtb: *const u8) -> (bool, u32) {
  if dtb == ptr::null() {
    return (false, 0);
  }

  unsafe {
    let magic = u32::from_be(*(dtb as *const u32));

    if magic == 0xd00dfeed {
      let size = u32::from_be(*(dtb.offset(4) as *const u32));
      return (true, size);
    }
  }

  return (false, 0);
}
