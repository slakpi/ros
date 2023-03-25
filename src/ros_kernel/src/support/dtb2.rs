//! Device Tree Utilities
//! https://devicetree-specification.readthedocs.io/en/stable/index.html

use super::bits;
use core::{cmp, slice, str};

const FDT_BEGIN_NODE: u32 = 0x1;
const FDT_END_NODE: u32 = 0x2;
const FDT_PROP: u32 = 0x3;
const FDT_NOOP: u32 = 0x4;
const FDT_END: u32 = 0x9;
const FDT_MAGIC: u32 = 0xd00dfeed;
const FDT_MAX_SIZE: usize = 64 * 1024 * 1024;
const FDT_WORD_BYTES: usize = (u32::BITS / 8) as usize;
const FDT_HEADER_SIZE: usize = FDT_WORD_BYTES * 8;
const FDT_MAX_CELL_COUNT: u32 = (usize::BITS / 8) as u32;

/// Error value for DTB operations.
pub enum DtbError {
  NotADtb,
  InvalidDtb,
}

/// A lightweight pointer to a location in a DTB that also provides methods to
/// read DTB primitives. DtbCursors are trivially copyable to save/restore
/// locations.
#[derive(Copy, Clone)]
pub struct DtbCursor {
  loc: usize,
}

impl DtbCursor {
  /// Create a new cursor.
  ///
  /// # Parameters
  ///
  /// * `loc` - The current location of the cursor.
  ///
  /// # Returns
  ///
  /// A new cursor.
  fn new(loc: usize) -> Self {
    DtbCursor { loc }
  }
}

/// DTB property header.
pub struct DtbPropertyHeader {
  pub size: usize,
  pub name_offset: usize,
}

/// DTB reader.
pub struct DtbReader<'blob> {
  dtb: &'blob [u8],
  dt_struct_offset: usize,
  dt_strings_offset: usize,
  _mem_rsv_map_offset: usize,
  _version: u32,
  _last_comp_version: u32,
  _boot_cpuid_phys: u32,
  _dt_strings_size: usize,
  _dt_struct_size: usize,
  addr_cells: u32,
  size_cells: u32,
}

impl<'blob> DtbReader<'blob> {
  /// Fast check to verify a DTB blob.
  ///
  /// # Parameters
  ///
  /// * `blob` - The address of the DTB blob.
  ///
  /// # Returns
  ///
  /// The total size of the DTB or a DtbError value.
  pub fn check_dtb(blob: usize) -> Result<usize, DtbError> {
    if blob == 0 {
      return Err(DtbError::NotADtb);
    }

    let dtb = blob as *const u32;
    let magic = unsafe { u32::from_be(*dtb) };

    if magic != FDT_MAGIC {
      return Err(DtbError::NotADtb);
    }

    let total_size = unsafe { u32::from_be(*dtb.add(1)) } as usize;

    if total_size < FDT_HEADER_SIZE || total_size > FDT_MAX_SIZE {
      return Err(DtbError::InvalidDtb);
    }

    Ok(total_size)
  }

  /// Create a new DTB reader.
  ///
  /// # Parameters
  ///
  /// * `blob` - The pointer to the DTB blob.
  ///
  /// # Returns
  ///
  /// A new DTB reader if the blob is a valid DTB, otherwise None.
  pub fn new(blob: usize) -> Option<Self> {
    let total_size = DtbReader::check_dtb(blob as usize).ok()?;
    let base_ptr = blob as *const u8;
    let mut cursor = DtbCursor::new(FDT_WORD_BYTES * 2);
    let mut dtb = DtbReader {
      dtb: unsafe { slice::from_raw_parts(base_ptr, total_size) },
      dt_struct_offset: 0,
      dt_strings_offset: 0,
      _mem_rsv_map_offset: 0,
      _version: 0,
      _last_comp_version: 0,
      _boot_cpuid_phys: 0,
      _dt_strings_size: 0,
      _dt_struct_size: 0,
      addr_cells: 0,
      size_cells: 0,
    };

    dtb.dt_struct_offset = dtb.get_u32(&mut cursor)? as usize;
    dtb.dt_strings_offset = dtb.get_u32(&mut cursor)? as usize;
    dtb._mem_rsv_map_offset = dtb.get_u32(&mut cursor)? as usize;
    dtb._version = dtb.get_u32(&mut cursor)?;
    dtb._last_comp_version = dtb.get_u32(&mut cursor)?;
    dtb._boot_cpuid_phys = dtb.get_u32(&mut cursor)?;
    dtb._dt_strings_size = dtb.get_u32(&mut cursor)? as usize;
    dtb._dt_struct_size = dtb.get_u32(&mut cursor)? as usize;

    let mut root = dtb.get_root_node()?;
    _ = dtb.get_null_terminated_u8_slice(&mut root)?;
    dtb.skip_and_align(&mut root, 1);

    while let Some(header) = dtb.get_next_property(&mut root) {
      let name = dtb.get_slice_from_string_table(header.name_offset).unwrap();

      if "#address-cells".as_bytes().cmp(name) == cmp::Ordering::Equal {
        dtb.addr_cells = dtb.get_u32(&mut root)?;
      } else if "#size-cells".as_bytes().cmp(name) == cmp::Ordering::Equal {
        dtb.size_cells = dtb.get_u32(&mut root)?;
      } else {
        dtb.skip_and_align(&mut root, header.size);
      }
    }

    if dtb.addr_cells < 1 || dtb.addr_cells > FDT_MAX_CELL_COUNT {
      return None;
    }

    if dtb.size_cells < 1 || dtb.size_cells > FDT_MAX_CELL_COUNT {
      return None;
    }

    Some(dtb)
  }

  /// Get a new cursor positioned at the start of the root node.
  ///
  /// # Returns
  ///
  /// A new cursor positioned after the FDT_BEGIN_NODE marker of the root node,
  /// or None if the root node is not found.
  pub fn get_root_node(&self) -> Option<DtbCursor> {
    let mut cursor = DtbCursor::new(self.dt_struct_offset);
    let marker = self.get_u32(&mut cursor)?;

    if marker != FDT_BEGIN_NODE {
      return None;
    }

    Some(cursor)
  }

  /// Find the specified child of the node pointed to by the cursor. Only
  /// immediate children are considered. If the child is found, the cursor
  /// returned will be positioned just after the node's name.
  ///
  /// # Parameters
  ///
  /// * `cursor` - A cursor pointing to a node.
  /// * `child_name` - The name of the child to find.
  ///
  /// # Assumptions
  ///
  /// The cursor is assumed to be positioned just after the FDT_BEGIN_NODE
  /// marker.
  ///
  /// # Returns
  ///
  /// A cursor if the child is found, otherwise None.
  pub fn find_child_node(
    &self,
    cursor: &DtbCursor,
    child_name: &str
  ) -> Option<DtbCursor> {
    let child_name_bytes = child_name.as_bytes();
    let mut tmp_cursor = *cursor;
    let mut depth = 0;

    loop {
      let name = self.get_null_terminated_u8_slice(&mut tmp_cursor)?;
      self.skip_and_align(&mut tmp_cursor, 1);

      // If we are at a depth of 1 (immediate child of the node pointed to by
      // cursor), compare the name. If we found the node, update the cursor and
      // return Ok.
      if depth == 1 && child_name_bytes.cmp(name) == cmp::Ordering::Equal {
        return Some(tmp_cursor);
      }

      let mut marker = self.consume_node_properties(&mut tmp_cursor).ok()?;

      loop {
        match marker {
          FDT_BEGIN_NODE => {
            depth += 1;
            break;
          },
          FDT_END_NODE => {
            if depth == 0 {
              return None;
            }

            depth -= 1;
          },
          FDT_NOOP => {},
          _ => return None,
        }

        marker = self.get_u32(&mut tmp_cursor)?;
      }
    }
  }

  /// Skips a node's properties.
  ///
  /// # Parameters
  ///
  /// * `cursor` - The cursor to advance.
  ///
  /// # Assumptions
  ///
  /// Assumes the cursor is positioned just after the node's name.
  ///
  /// # Returns
  ///
  /// Ok with the marker found if FDT_BEGIN_NODE or FDT_END_NODE is found,
  /// otherwise a DtbError.
  fn consume_node_properties(&self, cursor: &mut DtbCursor) -> Result<u32, DtbError> {
    loop {
      let marker = self.get_u32(cursor).ok_or(DtbError::InvalidDtb)?;

      match marker {
        FDT_PROP => {},
        FDT_BEGIN_NODE | FDT_END_NODE => return Ok(marker),
        FDT_NOOP => continue,
        _ => return Err(DtbError::InvalidDtb),
      }

      // Consume property header (size and name offset), then skip the property
      // value.
      let prop_size = self.get_u32(cursor).ok_or(DtbError::InvalidDtb)?;
      _ = self.get_u32(cursor).ok_or(DtbError::InvalidDtb)?;
      self.skip_and_align(cursor, prop_size as usize);
    }
  }

  /// Skip past a number of bytes and align the new location. Useful shortcut
  /// shortcut for skipping past a property, or the null terminator of a string,
  /// and any padding after.
  ///
  /// # Parameters
  ///
  /// * `cursor` - The cursor to advance.
  /// * `skip_bytes` - The number of bytes to skip.
  ///
  /// # Details
  ///
  /// If skipping the specified number of bytes would place the cursor past the
  /// end of the DTB, the cursor is positioned at the end of the DTB and is no
  /// longer valid.
  ///
  /// Otherwise, the cursor's position is updated by adding the number of bytes
  /// to the location and then aligning the new position on a DTB word boundary.
  pub fn skip_and_align(&self, cursor: &mut DtbCursor, skip_bytes: usize) {
    let len = self.dtb.len();
    let offset = cmp::min(len - cursor.loc, skip_bytes);

    cursor.loc = if cursor.loc + offset > len - FDT_WORD_BYTES {
      len
    } else {
      bits::align_up(cursor.loc + offset, FDT_WORD_BYTES)
    };
  }

  /// Read a 32-bit integer from the DTB at the position pointed to by the
  /// cursor. Advances the cursor by 32-bits. If a 32-bit integer could not be
  /// read, the cursor will not be repositioned.
  ///
  /// # Parameters
  ///
  /// * `cursor` - Cursor pointing to the location to read.
  ///
  /// # Returns
  ///
  /// The 32-bit integer at the current position or None if there are not at
  /// least 32-bits remaining in the DTB.
  pub fn get_u32(&self, cursor: &mut DtbCursor) -> Option<u32> {
    if cursor.loc > self.dtb.len() - FDT_WORD_BYTES {
      return None;
    }

    Some(self.get_u32_unchecked(cursor))
  }

  /// Internal helper to read a 32-bit integer.
  ///
  /// # Parameters
  ///
  /// * `cursor` - Cursor pointing to the location to read.
  ///
  /// # Details
  ///
  /// Assumes that the caller has already verified that 32-bits remain after the
  /// position pointed to by the cursor.
  fn get_u32_unchecked(&self, cursor: &mut DtbCursor) -> u32 {
    let end_loc = cursor.loc + FDT_WORD_BYTES;
    let bytes: &[u8; FDT_WORD_BYTES] = self.dtb[cursor.loc..end_loc].try_into().unwrap();
    let ret = u32::from_be_bytes(*bytes);
    cursor.loc = end_loc;
    ret
  }

  /// Read the property header of the next property after the position pointed
  /// to by the cursor.
  ///
  /// # Parameters
  ///
  /// * `cursor` - Cursor pointing to the location to read.
  ///
  /// # Assumptions
  ///
  /// Assumes the cursor is currently positioned just after the name of a node
  /// or just after the previous property's data.
  ///
  /// # Returns
  ///
  /// Returns the next property's header or None if a property is not found.
  pub fn get_next_property(&self, cursor: &mut DtbCursor) -> Option<DtbPropertyHeader> {
    loop {
      let marker = self.get_u32(cursor)?;
  
      match marker {
        FDT_PROP => {}
        FDT_NOOP => continue,
        _ => {
          cursor.loc -= FDT_WORD_BYTES;
          return None;
        },
      }
  
      return Some(DtbPropertyHeader {
        size: self.get_u32(cursor)? as usize,
        name_offset: self.get_u32(cursor)? as usize,
      });
    }
  }

  /// Get the size of a reg property value. The total size of a reg value
  /// depends on the platform and the cell count configuration.
  ///
  /// # Returns
  ///
  /// The total size of a reg property value.
  pub fn get_reg_size(&self) -> usize {
    (FDT_WORD_BYTES * self.addr_cells as usize) + (FDT_WORD_BYTES * self.size_cells as usize)
  }

  /// Read a reg value from the DTB as the position pointed to by the cursor.
  /// Advances the cursor by the total size of the reg value if the reg value
  /// could be read.
  ///
  /// https://devicetree-specification.readthedocs.io/en/stable/devicetree-basics.html#reg
  ///
  /// # Parameters
  ///
  /// * `cursor` - Cursor pointing to the location to read.
  ///
  /// # Returns
  ///
  /// A tuple with the address and size values or None if a reg value could not
  /// be read.
  pub fn get_reg(&self, cursor: &mut DtbCursor) -> Option<(usize, usize)> {
    let count = self.get_reg_size();

    if cursor.loc > self.dtb.len() - count {
      return None;
    }

    let mut addr = 0usize;
    let mut size = 0usize;

    for _ in 0..self.addr_cells {
      addr <<= FDT_WORD_BYTES;
      addr |= self.get_u32_unchecked(cursor) as usize;
    }

    for _ in 0..self.size_cells {
      size <<= FDT_WORD_BYTES;
      size |= self.get_u32_unchecked(cursor) as usize;
    }

    Some((addr, size))
  }

  /// Get the size of a range property value. The total size of a reg value
  /// depends on the platform and the cell count configuration.
  ///
  /// # Returns
  ///
  /// The total size of a reg property value.
  pub fn get_range_size(&self) -> usize {
    (FDT_WORD_BYTES * self.addr_cells as usize * 2) + (FDT_WORD_BYTES * self.size_cells as usize)
  }

  /// Read a range value from the DTB as the position pointed to by the cursor.
  /// Advances the cursor by the total size of the range value if the range
  /// value could be read.
  ///
  /// https://devicetree-specification.readthedocs.io/en/stable/devicetree-basics.html#ranges
  ///
  /// # Parameters
  ///
  /// * `cursor` - Cursor pointing to the location to read.
  ///
  /// # Returns
  ///
  /// A tuple with the child address, parent address, and size values or None if
  /// a range value could not be read.
  pub fn get_range(&self, cursor: &mut DtbCursor) -> Option<(usize, usize, usize)> {
    let count = self.get_range_size();

    if cursor.loc > self.dtb.len() - count {
      return None;
    }

    let mut child_addr = 0usize;
    let mut parent_addr = 0usize;
    let mut size = 0usize;

    for _ in 0..self.addr_cells {
      child_addr <<= FDT_WORD_BYTES;
      child_addr |= self.get_u32_unchecked(cursor) as usize;
    }

    for _ in 0..self.addr_cells {
      parent_addr <<= FDT_WORD_BYTES;
      parent_addr |= self.get_u32_unchecked(cursor) as usize;
    }

    for _ in 0..self.size_cells {
      size <<= FDT_WORD_BYTES;
      size |= self.get_u32_unchecked(cursor) as usize;
    }

    Some((child_addr, parent_addr, size))
  }

  /// Gets a slice from the position pointed to by the cursor. The slice will
  /// contain at most the specified number of bytes.
  ///
  /// # Parameters
  ///
  /// * `cursor` - Cursor pointing to the location to read.
  /// * `size` - The maximum number of bytes to retrieve.
  ///
  /// # Returns
  ///
  /// A slice containing at most the specified number of bytes. If the slice
  /// contains fewer bytes, the cursor will be positioned at the end of the DTB.
  pub fn get_u8_slice(&self, cursor: &mut DtbCursor, size: usize) -> Option<&'blob [u8]> {
    let len = self.dtb.len();
    let end = cursor.loc + cmp::min(len - cursor.loc, size);
    let ret = &self.dtb[cursor.loc..end];
    cursor.loc = end;

    Some(ret)
  }

  /// Gets a null-terminated slice starting at the position pointed to by the
  /// cursor. The cursor will be advanced to the null-terminator. The caller
  /// should use `skip_and_align` to skip the 1-byte terminator and align the
  /// cursor. If a null-terminator is not found before the end of the DTB, the
  /// cursor will not be repositioned.
  ///
  /// # Parameters
  ///
  /// * `cursor` - Cursor pointing to the location to read.
  ///
  /// # Returns
  ///
  /// A slice containing the bytes up to, but NOT including, the null-terminator
  /// if a null-terminated string was found, otherwise None.
  pub fn get_null_terminated_u8_slice(&self, cursor: &mut DtbCursor) -> Option<&'blob [u8]> {
    let mut end = cursor.loc;
    let len = self.dtb.len();

    while end < len {
      if self.dtb[end] == 0 {
        break;
      }

      end += 1;
    }

    // We did not actually find a null-terminator, this is invalid.
    if end == len {
      return None;
    }

    // Return a slice excluding the null-terminator and leave the cursor on the
    // null terminator.
    let ret = &self.dtb[cursor.loc..end];
    cursor.loc = end;

    Some(ret)
  }

  /// Returns a slice from the specified position in the string table.
  ///
  /// # Parameters
  ///
  /// * `str_offset` - The byte offset into the string table.
  ///
  /// # Returns
  ///
  /// A slice containing the string if a null-terminated string was found at the
  /// specified offset, otherwise None.
  pub fn get_slice_from_string_table(&self, str_offset: usize) -> Option<&'blob [u8]> {
    let mut cursor = DtbCursor::new(self.dt_strings_offset + str_offset);
    self.get_null_terminated_u8_slice(&mut cursor)
  }
}
