use core::{cmp, ptr, slice};

/// https://devicetree-specification.readthedocs.io/en/stable/index.html

const FDT_BEGIN_NODE: u32 = 0x1;
const FDT_END_NODE: u32 = 0x2;
const FDT_PROP: u32 = 0x3;
const FDT_NOOP: u32 = 0x4;
const FDT_END: u32 = 0x9;
const FDT_WORD_BYTES: u32 = u32::BITS / 8;

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
///          only verifies it has not read past the declared length of the DTB,
///          it does not have any semantic knowledge of the DTB structure.
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
    debug_assert!(total_size > FDT_WORD_BYTES);

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

  /// @fn skip_and_align(&mut self, skip_bytes: u32)
  /// @brief Skip past a number of bytes and align the new location. Useful
  ///        shortcut for skipping past a property or the null terminator of a
  ///        string.
  /// @param[in] skip_bytes The number of bytes to skip before alignment.
  pub fn skip_and_align(&mut self, skip_bytes: u32) {
    let offset = cmp::min(self.total_size - self.cur_loc, skip_bytes);

    let new_loc = if self.cur_loc + offset > self.total_size - FDT_WORD_BYTES {
      self.total_size
    } else {
      (self.cur_loc + offset + (FDT_WORD_BYTES - 1)) & (-(FDT_WORD_BYTES as i32) as u32)
    };

    self.set_loc(new_loc);
  }

  /// @fn align_loc(&mut self)
  /// @brief Align the current location on a u32 boundary.
  pub fn align_loc(&mut self) {
    self.skip_and_align(0);
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

  pub fn unget_u8(&mut self) {
    if self.cur_loc < 1 {
      return;
    }

    self.set_loc(self.cur_loc - 1);
  }

  /// @fn fn get_u8_slice<'cursor>(&mut self, size: u32) -> Option<&'cursor [u8]>
  /// @brief   Construct a slice referencing the next @a size u8's.
  /// @param[in] size The requested length of the slice.
  /// @returns A slice or None if @a size overruns the DTB.
  pub fn get_u8_slice<'cursor>(&mut self, size: u32) -> Option<&'cursor [u8]> {
    debug_assert!(size <= isize::MAX as u32);

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

  /// @fn get_u8_slice_null_terminated<'cursor>(&mut self) -> Option<&'cursor [u8]>
  /// @brief   Create a slice from a null terminated string.
  /// @details Use @a get_u8_slice when the size is already known. The cursor
  ///          will point to the null terminator when successful.
  /// @returns A slice or None if a null terminator is not found.
  pub fn get_u8_slice_null_terminated<'cursor>(&mut self) -> Option<&'cursor [u8]> {
    let mut p = self.cur_ptr;
    let mut e = self.cur_loc;

    // Find the null terminator.
    unsafe {
      while e < self.total_size {
        if *p == 0 {
          break;
        }

        p = p.add(1);
        e += 1;
      }
    }

    // We did not actually find a null terminator, this is invalid.
    if e == self.total_size {
      return None;
    }

    debug_assert!(e - self.cur_loc <= isize::MAX as u32);

    // Return a slice excluding the null terminator and leave the cursor on the
    // null terminator.
    let ret = unsafe { slice::from_raw_parts(self.cur_ptr, (e - self.cur_loc) as usize) };
    self.cur_ptr = p;
    self.cur_loc = e;

    Some(ret)
  }

  /// @fn get_u32(&mut self) -> Option<u32>
  /// @brief   Read the next u32 from the DTB.
  /// @returns The next u32 or None if the end of the DTB has been reached.
  pub fn get_u32(&mut self) -> Option<u32> {
    if self.cur_loc > self.total_size - FDT_WORD_BYTES {
      return None;
    }

    let ret = unsafe { u32::from_be(*(self.cur_ptr as *const u32)) };
    self.cur_ptr = unsafe { self.cur_ptr.add(FDT_WORD_BYTES as usize) };
    self.cur_loc += FDT_WORD_BYTES;

    Some(ret)
  }

  pub fn unget_u32(&mut self) {
    if self.cur_loc < FDT_WORD_BYTES {
      return;
    }

    self.set_loc(self.cur_loc - FDT_WORD_BYTES);
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

    let count = (FDT_WORD_BYTES * addr_cells) + (FDT_WORD_BYTES * size_cells);

    if self.cur_loc > self.total_size - count {
      return None;
    }

    let mut addr = 0usize;
    let mut size = 0usize;

    for _ in 0..addr_cells {
      addr <<= FDT_WORD_BYTES;
      let word = self.get_u32()?;
      addr |= word as usize;
    }

    for _ in 0..size_cells {
      size <<= FDT_WORD_BYTES;
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

/// @fn scan_dtb(blob: usize) -> Result<(), ()>
///
pub fn scan_dtb(blob: usize) -> Result<(), ()> {
  let total_size = match check_dtb(blob) {
    Ok(total_size) => total_size,
    Err(_) => return Err(()),
  };

  let mut cursor = DtbCursor::new(blob as *const u8, total_size);
  cursor.set_loc(FDT_WORD_BYTES * 2); // Skip magic and total size.

  let hdr = DtbHeader::new(&mut cursor);
  cursor.set_loc(hdr.dt_struct_offset); // Skip to the root node.

  let (addr_cells, size_cells) = match scan_root_none(&hdr, &mut cursor) {
    Some(sizes) => sizes,
    None => return Err(()),
  };

  Ok(())
}

/// @fn scan_root_none<'cursor>(hdr: &DtbHeader, cursor: &'cursor mut DtbCursor) -> Option<(u32, u32)>
///
fn scan_root_none<'cursor>(hdr: &DtbHeader, cursor: &'cursor mut DtbCursor) -> Option<(u32, u32)> {
  // Verify we are at the start of a node.
  let begin = cursor.get_u32()?;

  if begin != FDT_BEGIN_NODE {
    return None;
  }

  // Verify zero-length name.
  let name = cursor.get_u8_slice_null_terminated()?;

  if name.len() != 0 {
    return None;
  }

  cursor.skip_and_align(1);

  let mut addr_cells = 0;
  let mut size_cells = 0;

  loop {
    let (prop_size, name_offset) = match move_to_next_property(hdr, cursor) {
      Some(tuple) => tuple,
      _ => break,
    };

    let prop_name = get_string_from_table(name_offset, cursor)?;

    if "#address_cells".as_bytes() == prop_name {
      addr_cells = cursor.get_u32()?;
    } else if "#size_cells".as_bytes() == prop_name {
      size_cells = cursor.get_u32()?;
    } else {
      cursor.skip_and_align(prop_size);
    }

    if addr_cells > 0 && size_cells > 0 {
      return Some((addr_cells, size_cells));
    }
  }

  None
}

fn move_to_next_property(hdr: &DtbHeader, cursor: &mut DtbCursor) -> Option<(u32, u32)> {
  loop {
    let begin = cursor.get_u32()?;

    match begin {
      // Beginning of next property.
      FDT_PROP => {}
      // Beginning of child node, reset the cursor and break.
      FDT_BEGIN_NODE => {
        cursor.unget_u32();
        break;
      }
      // End of node, just break leaving the cursor past the end of node.
      FDT_END_NODE => break,
      // Noops are allowed as necessary and can be skipped.
      FDT_NOOP => continue,
      // Ignore anything else.
      _ => break,
    }

    let prop_size = cursor.get_u32()?;
    let name_offset = cursor.get_u32()? + hdr.dt_strings_offset;
    return Some((prop_size, name_offset));
  }

  None
}

fn get_string_from_table<'cursor>(
  str_offset: u32,
  cursor: &'cursor mut DtbCursor,
) -> Option<&'cursor [u8]> {
  let old_loc = cursor.cur_loc;
  cursor.set_loc(str_offset);

  let ret = cursor.get_u8_slice_null_terminated()?;
  cursor.set_loc(old_loc);

  Some(ret)
}
