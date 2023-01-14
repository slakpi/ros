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

  let total_size = unsafe { u32::from_be(*dtb.add(1)) };

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

  /// @fn get_u8(&mut self) -> Option<u8>
  /// @brief   Read the next u8 from the DTB.
  /// @returns The next u8 or None if the end of the DTB has been reached.
  pub fn get_u8(&mut self) -> Option<u8> {
    if self.cur_loc > self.total_size - 1 {
      return None;
    }

    let ret = unsafe { *self.cur_ptr };
    self.cur_ptr = unsafe { self.cur_ptr.add(1) };
    self.cur_loc += 1;
    
    Some(ret)
  }

  /// @fn get_u8_slice(&mut self, size: u32) -> Option<&[u8]>
  /// @brief   Construct a slice referencing the next @a size u8's.
  /// @param[in] size The requested length of the slice.
  /// @returns A slice or None if @a size overruns the DTB.
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

  // pub fn get_u8_slice_null_terminated(&mut self) -> Option<&[u8]> {
  //   let mut p = self.cur_ptr;
  //   let mut e = self.cur_loc;

  //   unsafe {
  //     while e < self.total_size {
  //       if *p == 0 {
  //         break;
  //       }

  //       p = p.add(1);
  //       e += 1;
  //     }
  //   }

  //   let ret = unsafe { slice::from_raw_parts(self.cur_ptr, e - self.cur_loc) };

    

  //   self.cur_ptr = p;
  //   self.cur_loc = e;
  // }

  /// @fn get_u32(&mut self) -> Option<u32>
  /// @brief   Read the next u32 from the DTB.
  /// @returns The next u32 or None if the end of the DTB has been reached.
  pub fn get_u32(&mut self) -> Option<u32> {
    if self.cur_loc > self.total_size - 4 {
      return None;
    }

    let ret = unsafe { u32::from_be(*(self.cur_ptr as *const u32)) };
    self.cur_ptr = unsafe { self.cur_ptr.add(4) };
    self.cur_loc += 4;
    
    Some(ret)
  }

  /// @fn get_reg(&mut self, addr_cells: u32, size_cells: u32) -> Option<(usize, usize)>
  /// @brief   Read the next reg pair.
  /// @param[in] addr_cells The number of u32 words in an address.
  /// @param[in] size_cells The number of u32 words in a range size.
  /// @returns A tuple with the base address and size, or None if the reg
  ///          overruns the DTB.
  pub fn get_reg(&mut self, addr_cells: u32, size_cells: u32) -> Option<(usize, usize)> {
    if addr_cells > 2 || size_cells > 2 {
      return None;
    }

    let count = (4 * addr_cells) + (4 * size_cells);

    if self.cur_loc > self.total_size - count {
      return None;
    }

    let mut addr = 0usize;
    let mut size = 0usize;

    for _ in 0..addr_cells {
      addr <<= 4;
      let word = self.get_u32()?;
      addr |= word as usize;
    }

    for _ in 0..size_cells {
      size <<= 4;
      let word = self.get_u32()?;
      size |= word as usize;
    }

    Some((addr, size))
  }
}

struct DtbHeader {
  dt_struct_offset: u32,
  dt_strings_offset: u32,
  mem_rsv_map_offset: u32,
  version: u32,
  last_comp_version: u32,
  boot_cpuid_phys: u32,
  dt_strings_size: u32,
  dt_struct_size: u32,
}

impl DtbHeader {
  pub fn new(cursor: &mut DtbCursor) -> Self {
    DtbHeader {
      dt_struct_offset: cursor.get_u32().unwrap(),
      dt_strings_offset: cursor.get_u32().unwrap(),
      mem_rsv_map_offset: cursor.get_u32().unwrap(),
      version: cursor.get_u32().unwrap(),
      last_comp_version: cursor.get_u32().unwrap(),
      boot_cpuid_phys: cursor.get_u32().unwrap(),
      dt_strings_size: cursor.get_u32().unwrap(),
      dt_struct_size: cursor.get_u32().unwrap(),
    }
  }
}

pub fn scan_dtb(blob: usize) {
  let total_size = match check_dtb(blob) {
    Ok(total_size) => total_size,
    Err(_) => return,
  };

  let mut cursor = DtbCursor::new(blob as *const u8, total_size);
  cursor.set_loc(8); // Skip magic and total size.

  let hdr = DtbHeader::new(&mut cursor);
  cursor.set_loc(hdr.dt_struct_offset); // Skip to the root node.


}
