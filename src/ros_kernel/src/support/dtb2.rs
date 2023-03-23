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

pub enum DtbError {
  NotADtb,
  InvalidDtb,
}

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
  fn new(base_ptr: *const u8, loc: usize, total_size: usize) -> Self {
    debug_assert!(loc < total_size);
    debug_assert!(total_size > FDT_WORD_BYTES);

    DtbCursor {
      dtb: unsafe { slice::from_raw_parts(base_ptr, total_size as usize) },
      loc: loc as usize,
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

  pub fn get_u32(&mut self) -> Option<u32> {
    if self.loc > self.end_loc - FDT_WORD_BYTES {
      return None;
    }

    self.loc += FDT_WORD_BYTES;

    Some(unsafe { u32::from_be(*(ptr as *const u32)) })
  }

  pub fn get_reg(&mut self, addr_cells: u32, size_cells: u32) -> Option<(usize, usize)> {
    if addr_cells < 1 || addr_cells > 2 {
      return None;
    }

    if size_cells < 1 || size_cells > 2 {
      return None;
    }

    let count = (FDT_WORD_BYTES * addr_cells) + (FDT_WORD_BYTES * size_cells);

    if self.loc > self.end_loc - count {
      return None;
    }

    let mut addr = 0usize;
    let mut size = 0usize;

    for _ in 0..addr_cells {
      addr <<= FDT_WORD_BYTES;
      addr |= self.get_u32()? as usize;
    }

    for _ in 0..size_cells {
      size <<= FDT_WORD_BYTES;
      size |= self.get_u32()? as usize;
    }

    Some((addr, size))
  }
}

pub struct Dtb {
  base_ptr: *const u8,
  total_size: u32,
  dt_struct_offset: u32,
  dt_strings_offset: u32,
  _mem_rsv_map_offset: u32,
  _version: u32,
  _last_comp_version: u32,
  _boot_cpuid_phys: u32,
  _dt_strings_size: u32,
  _dt_struct_size: u32,
}

impl Dtb {
  pub fn new(blob: u32) -> Option<Self> {
    None
  }
}
