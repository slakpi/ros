//! Device Tree Utilities
//! https://devicetree-specification.readthedocs.io/en/stable/index.html

use super::bits;
use core::{cmp, ops, slice};

const FDT_BEGIN_NODE: u32 = 0x1;
const FDT_END_NODE: u32 = 0x2;
const FDT_PROP: u32 = 0x3;
const FDT_NOOP: u32 = 0x4;
const FDT_END: u32 = 0x9;
const FDT_CELL_COUNTS: ops::Range<u32> = 1..3;
const FDT_MAGIC: u32 = 0xd00dfeed;
const FDT_MAX_SIZE: usize = 64 * 1024 * 1024;
const FDT_WORD_BYTES: usize = (u32::BITS / 8) as usize;
const FDT_HEADER_SIZE: usize = FDT_WORD_BYTES * 8;
const FDT_MAX_CELL_COUNT: usize = (usize::BITS / 8) as usize;

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
  /// * `blob` - The 32-bit pointer to the DTB blob.
  ///
  /// # Returns
  ///
  /// A new DTB reader if the blob is a valid DTB, otherwise None.
  pub fn new(blob: u32) -> Option<Self> {
    let total_size = DtbReader::check_dtb(blob as usize).ok()?;
    let base_ptr = blob as *const u8;
    let mut cursor = DtbCursor::new(0);
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
    };

    dtb.dt_struct_offset = dtb.get_u32(&mut cursor)? as usize;
    dtb.dt_strings_offset = dtb.get_u32(&mut cursor)? as usize;
    dtb._mem_rsv_map_offset = dtb.get_u32(&mut cursor)? as usize;
    dtb._version = dtb.get_u32(&mut cursor)?;
    dtb._last_comp_version = dtb.get_u32(&mut cursor)?;
    dtb._boot_cpuid_phys = dtb.get_u32(&mut cursor)?;
    dtb._dt_strings_size = dtb.get_u32(&mut cursor)? as usize;
    dtb._dt_struct_size = dtb.get_u32(&mut cursor)? as usize;

    Some(dtb)
  }

  /// Get a new cursor positioned at the start of the root node.
  ///
  /// # Returns
  ///
  /// A new cursor positioned after the FDT_BEGIN_NODE marker of the root node,
  /// or None if the root node is not found.
  pub fn get_root_node(&self) -> Option<DtbCursor> {
    let mut cursor = DtbCursor::new(FDT_HEADER_SIZE);
    let marker = self.get_u32(&mut cursor)?;

    if marker != FDT_BEGIN_NODE {
      return None;
    }

    Some(cursor)
  }

  /// Moves the cursor to the specified child of the node pointed to by the
  /// cursor. Only immediate children are considered. If the child is not found,
  /// the cursor is not moved. If the child is found, the cursor will be
  /// positioned after the FDT_BEGIN_NODE marker.
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
  /// True if the child is found, false otherwise.
  pub fn move_cursor_to_child(&self, cursor: &mut DtbCursor, child_name: &str) -> bool {
    let mut tmp_cursor = cursor;
    let mut depth = 0;

    false
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

  /// Read a reg value from the DTB as the position pointed to by the cursor.
  /// Advances the cursor by the total size of the reg value. The total size of
  /// the reg value depends on the platform and the cell count configuration. If
  /// a reg value could not be read, the cursor will not be repositioned.
  ///
  /// # Parameters
  ///
  /// * `cursor` - Cursor pointing to the location to read.
  /// * `addr_cells` - The number of 32-bit cells in an address.
  /// * `size_cells` - The number of 32-bit cells in a size.
  ///
  /// # Details
  ///
  /// The cell counts are obtained from the DTB header.
  ///
  /// # Returns
  ///
  /// A tuple with the address and size values or None if the cell counts are
  /// invalid.
  pub fn get_reg(
    &self,
    cursor: &mut DtbCursor,
    addr_cells: u32,
    size_cells: u32
  ) -> Option<(usize, usize)> {
    let addr_cells = addr_cells as usize;
    let size_cells = size_cells as usize;

    if addr_cells < 1 || addr_cells > FDT_MAX_CELL_COUNT {
      return None;
    }

    if size_cells < 1 || size_cells > FDT_MAX_CELL_COUNT {
      return None;
    }

    let count = (FDT_WORD_BYTES * addr_cells) + (FDT_WORD_BYTES * size_cells);

    if cursor.loc > self.dtb.len() - count {
      return None;
    }

    let mut addr = 0usize;
    let mut size = 0usize;

    for _ in 0..addr_cells {
      addr <<= FDT_WORD_BYTES;
      addr |= self.get_u32_unchecked(cursor) as usize;
    }

    for _ in 0..size_cells {
      size <<= FDT_WORD_BYTES;
      size |= self.get_u32_unchecked(cursor) as usize;
    }

    Some((addr, size))
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
  /// cursor. If a null-terminator is not found before the end of the DTB, the
  /// cursor will not be repositioned.
  pub fn get_null_terminated_u8_slice(&self, cursor: &mut DtbCursor) -> Option<&'blob [u8]> {
    let mut end = cursor.loc;
    let len = self.dtb.len();

    while end < len {
      if self.dtb[end] == 0 {
        break;
      }

      end += 1;
    }

    // We did not actually find a null terminator, this is invalid.
    if end == len {
      return None;
    }

    // Return a slice excluding the null terminator and leave the cursor on the
    // null terminator.
    let ret = &self.dtb[cursor.loc..end];
    cursor.loc = end;

    Some(ret)
  }
}
