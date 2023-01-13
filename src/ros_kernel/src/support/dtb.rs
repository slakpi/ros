use core::{cmp, ptr, slice};

/// https://devicetree-specification.readthedocs.io/en/stable/flattened-format.html

const FDT_BEGIN_NODE: u32 = 0x1;
const FDT_END_NODE: u32 = 0x2;
const FDT_PROP: u32 = 0x3;
const FDT_NOOP: u32 = 0x4;
const FDT_END: u32 = 0x9;

/// @fn check_dtb(blob: usize) -> Result<u32, ()>
/// @brief   Fast check to verify the blob is a valid flat devicetree.
/// @returns Ok with the size of the devicetree or Err.
pub fn check_dtb(blob: usize) -> Result<u32, ()> {
  if blob == 0 {
    return Err(());
  }

  let dtb = blob as *const u32;
  let magic = unsafe { u32::from_be(*dtb) };

  if magic != 0xd00dfeed {
    return Err(());
  }

  let total_size = unsafe { u32::from_be(*dtb.offset(1)) };

  Ok(total_size)
}

/// @struct  DtbCursor
/// @brief   Handles raw access to a DTB.
/// @details The DtbCursor tracks the current location in the DTB and handles
///          reading raw u8's, u8 slices, u32's, and reg pairs. The DtbCursor
///          only verifies it has not read past the declared length of the DTB.
struct DtbCursor {
  base_ptr: *const u8,
  cur_ptr: *const u8,
  cur_loc: u32,
  total_size: u32,
}

impl DtbCursor {
  /// @fn new(ptr: *const u8, total_size: u32) -> Self
  /// @brief   Construct a new cursor with the given base pointer and total
  ///          size.
  /// @param[in] ptr        The base pointer of the DTB.
  /// @param[in] total_size The total size, in bytes, of the DTB.
  /// @returns A new DtbCursor set to an offset of 0 from the base pointer.
  pub fn new(ptr: *const u8, total_size: u32) -> Self {
    DtbCursor {
      base_ptr: ptr,
      cur_ptr: ptr,
      cur_loc: 0,
      total_size: total_size,
    }
  }

  /// @fn set_loc(&mut self, loc: u32)
  /// @brief   Move the cursor to an offset from the beginning of the DTB.
  /// @details The cursor will not move beyond the end of the DTB. If @a loc is
  ///          greater than the total size, the cursor will move to the end of
  ///          the DTB.
  /// @param[in] loc New offset, in bytes, from the beginning of the DTB.
  pub fn set_loc(&mut self, loc: u32) {
    let loc = cmp::min(loc, self.total_size);
    self.cur_ptr = unsafe { self.base_ptr.add(loc as usize) };
    self.cur_loc = loc;
  }

  pub fn get_u8(&mut self) -> Option<u8> {
    if self.cur_loc > self.total_size - 1 {
      return None;
    }

    let ret = unsafe { *self.cur_ptr };
    self.cur_ptr = unsafe { self.cur_ptr.add(1) };
    self.cur_loc += 1;
    
    Some(ret)
  }

  pub fn get_u8_slice(&mut self, size: u32) -> Option<&[u8]> {
    if size > self.total_size {
      return None;
    }

    if self.cur_loc > self.total_size - size {
      return None;
    }

    let ret = unsafe { slice::from_raw_parts(self.cur_ptr, size as usize) };
    self.cur_ptr = unsafe { self.cur_ptr.add(size as usize) };
    self.cur_loc += size;

    Some(ret)
  }

  pub fn get_u32(&mut self) -> Option<u32> {
    if self.cur_loc > self.total_size - 4 {
      return None;
    }

    let ret = unsafe { *(self.cur_ptr as *const u32) };
    self.cur_ptr = unsafe { self.cur_ptr.add(4) };
    self.cur_loc += 1;
    
    Some(ret)
  }

  pub fn get_reg(&mut self, cell_count: u32, size_count: u32) -> Option<(usize, usize)> {
    let mut addr = 0usize;
    let mut size = 0usize;

    for _ in 0..cell_count {
      addr <<= 4;
      let word = self.get_u32()?;
      addr |= word as usize;
    }

    for _ in 0..size_count {
      size <<= 4;
      let word = self.get_u32()?;
      size |= word as usize;
    }

    Some((addr, size))
  }
}

pub fn scan_dtb(blob: usize) {
  let total_size = match check_dtb(blob) {
    Ok(total_size) => total_size,
    Err(_) => return,
  };

  let cursor = DtbCursor::new(blob as *const u8, total_size);
}
