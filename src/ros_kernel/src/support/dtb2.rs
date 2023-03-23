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
const FDT_MAX_CELL_COUNT: usize = (usize::BITS / 8) as usize;

/// Error value for DTB operations.
pub enum DtbError {
  NotADtb,
  InvalidDtb,
}

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

  if total_size > FDT_MAX_SIZE {
    return Err(DtbError::InvalidDtb);
  }

  Ok(total_size)
}

pub struct DtbNode {
  loc: u32,
}

/// A lightweight pointer to a location in a DTB that also provides methods to
/// read DTB primitives. DtbCursors are trivially copyable to save/restore
/// locations.
#[derive(Copy, Clone)]
pub struct DtbCursor<'blob> {
  dtb: &'blob [u8],
  loc: usize,
}

impl<'blob> DtbCursor<'blob> {
  /// Create a new cursor.
  ///
  /// # Parameters
  ///
  /// * `ptr` - The base pointer of the DTB.
  /// * `loc` - The current location of the cursor.
  /// * `total_size` - The total size of the DTB.
  ///
  /// # Returns
  ///
  /// A new cursor.
  fn new(dtb: &'blob [u8], loc: usize) -> Self {
    debug_assert!(loc < dtb.len());

    DtbCursor {
      dtb,
      loc,
    }
  }

  /// Skip past a number of bytes and align the new location. Useful shortcut
  /// shortcut for skipping past a property, or the null terminator of a string,
  /// and any padding after.
  ///
  /// # Parameters
  ///
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
  pub fn skip_and_align(&mut self, skip_bytes: u32) {
    let end_loc = self.dtb.len();
    let offset = cmp::min(end_loc - self.loc, skip_bytes as usize);

    self.loc = if self.loc + offset > end_loc - FDT_WORD_BYTES {
      end_loc
    } else {
      bits::align_up(self.loc + offset, FDT_WORD_BYTES)
    };
  }

  /// Read a 32-bit integer from the DTB at the position pointed to by the
  /// cursor. Advances the cursor by 32-bits.
  ///
  /// # Returns
  ///
  /// The 32-bit integer at the current position or None if there are not at
  /// least 32-bits remaining in the DTB.
  pub fn get_u32(&mut self) -> Option<u32> {
    let end_loc = self.dtb.len();

    if self.loc > end_loc - FDT_WORD_BYTES {
      self.loc = end_loc;
      return None;
    }

    Some(self.get_u32_unchecked())
  }

  /// Internal helper to read a 32-bit integer.
  ///
  /// # Details
  ///
  /// Assumes that the caller has already verified that 32-bits remain after the
  /// position pointed to by the cursor.
  fn get_u32_unchecked(&mut self) -> u32 {
    let end = self.loc + FDT_WORD_BYTES;
    let bytes: &[u8; FDT_WORD_BYTES] = self.dtb[self.loc..end].try_into().unwrap();
    let ret = u32::from_be_bytes(*bytes);
    self.loc += FDT_WORD_BYTES;
    ret
  }

  /// Read a reg value from the DTB as the position pointed to by the cursor.
  /// Advances the cursor by the total size of the reg value. The total size of
  /// the reg value depends on the platform and the cell count configuration.
  ///
  /// # Parameters
  ///
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
  pub fn get_reg(&mut self, addr_cells: u32, size_cells: u32) -> Option<(usize, usize)> {
    let addr_cells = addr_cells as usize;
    let size_cells = size_cells as usize;

    if addr_cells < 1 || addr_cells > FDT_MAX_CELL_COUNT {
      return None;
    }

    if size_cells < 1 || size_cells > FDT_MAX_CELL_COUNT {
      return None;
    }

    let count = (FDT_WORD_BYTES * addr_cells) + (FDT_WORD_BYTES * size_cells);

    if self.loc > self.dtb.len() - count {
      return None;
    }

    let mut addr = 0usize;
    let mut size = 0usize;

    for _ in 0..addr_cells {
      addr <<= FDT_WORD_BYTES;
      addr |= self.get_u32_unchecked() as usize;
    }

    for _ in 0..size_cells {
      size <<= FDT_WORD_BYTES;
      size |= self.get_u32_unchecked() as usize;
    }

    Some((addr, size))
  }
}

pub struct Dtb<'blob> {
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

impl<'blob> Dtb<'blob> {
  pub fn new(blob: u32) -> Option<Self> {
    let total_size = check_dtb(blob as usize).ok()?;
    let base_ptr = blob as *const u8;
    let dtb = unsafe { slice::from_raw_parts(base_ptr, total_size) };
    let mut cursor = DtbCursor::new(dtb, 0);

    Some(Dtb {
      dtb,
      dt_struct_offset: cursor.get_u32()? as usize,
      dt_strings_offset: cursor.get_u32()? as usize,
      _mem_rsv_map_offset: cursor.get_u32()? as usize,
      _version: cursor.get_u32()?,
      _last_comp_version: cursor.get_u32()?,
      _boot_cpuid_phys: cursor.get_u32()?,
      _dt_strings_size: cursor.get_u32()? as usize,
      _dt_struct_size: cursor.get_u32()? as usize,
    })
  }

  pub fn get_root_node(&self) -> DtbCursor {
    DtbCursor::new(self.dtb, FDT_WORD_BYTES * 8)
  }
}
