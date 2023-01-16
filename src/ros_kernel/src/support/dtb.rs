use super::align;
use core::{cmp, ops, slice};

/// https://devicetree-specification.readthedocs.io/en/stable/index.html

const FDT_BEGIN_NODE: u32 = 0x1;
const FDT_END_NODE: u32 = 0x2;
const FDT_PROP: u32 = 0x3;
const FDT_NOOP: u32 = 0x4;
const FDT_END: u32 = 0x9;
const FDT_WORD_BYTES: u32 = u32::BITS / 8;
const FDT_CELL_COUNTS: ops::Range<u32> = 1..3;
const FDT_MAGIC: u32 = 0xd00dfeed;
const FDT_MAX_SIZE: u32 = 0x4000000; // 64 MB should be plenty.

/// @enum  DtbError
/// @brief DTB error codes.
pub enum DtbError {
  NotADtb,
  InvalidDtb,
}

/// @fn check_dtb(blob: usize) -> Result<u32, ()>
/// @brief   Fast check to verify the blob is a valid flat devicetree.
/// @details Verifies the magic value that should be at the beginning of the DTB
///          and verifies that the size is sane.
/// @returns Ok with the size of the devicetree or Err.
pub fn check_dtb(blob: usize) -> Result<u32, DtbError> {
  if blob == 0 {
    return Err(DtbError::NotADtb);
  }

  let dtb = blob as *const u32;
  let magic = unsafe { u32::from_be(*dtb) };

  if magic != FDT_MAGIC {
    return Err(DtbError::NotADtb);
  }

  let total_size = unsafe { u32::from_be(*dtb.add(1)) };

  if total_size > FDT_MAX_SIZE {
    return Err(DtbError::InvalidDtb);
  }

  Ok(total_size)
}

/// @struct  DtbCursor
/// @brief   Handles raw access to a DTB.
/// @details The DtbCursor tracks the current location in the DTB and handles
///          reading raw u8's, u8 slices, u32's, and reg pairs. The DtbCursor
///          only verifies it has not read past the declared length of the DTB,
///          it does not have any semantic knowledge of the DTB structure.
pub struct DtbCursor {
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
  fn new(ptr: *const u8, total_size: u32) -> Self {
    debug_assert!(total_size > FDT_WORD_BYTES);

    DtbCursor {
      base_ptr: ptr,
      cur_ptr: ptr,
      cur_loc: 0,
      total_size: total_size,
    }
  }

  /// @fn get_loc(&self) -> u32
  /// @brief   Get the current location in the blob.
  /// @returns The current location.
  pub fn get_loc(&self) -> u32 {
    self.cur_loc
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
  ///        shortcut for skipping past a property, or the null terminator of a
  ///        string, and any padding after.
  /// @param[in] skip_bytes The number of bytes to skip before alignment.
  pub fn skip_and_align(&mut self, skip_bytes: u32) {
    let offset = cmp::min(self.total_size - self.cur_loc, skip_bytes);

    let new_loc = if self.cur_loc + offset > self.total_size - FDT_WORD_BYTES {
      self.total_size
    } else {
      align::align_up(self.cur_loc + offset, FDT_WORD_BYTES)
    };

    self.set_loc(new_loc);
  }

  /// @fn align_loc(&mut self)
  /// @brief Align the current location on a u32 boundary.
  pub fn _align_loc(&mut self) {
    self.skip_and_align(0);
  }

  /// @fn get_u8(&mut self) -> Option<u8>
  /// @brief   Read the next u8 from the DTB.
  /// @returns The next u8 or None if the end of the DTB has been reached.
  pub fn _get_u8(&mut self) -> Option<u8> {
    if self.cur_loc > self.total_size - 1 {
      return None;
    }

    let ret = unsafe { *self.cur_ptr };
    self.cur_ptr = unsafe { self.cur_ptr.add(1) };
    self.cur_loc += 1;

    Some(ret)
  }

  /// @fn unget_u8(&mut self)
  /// @brief Move the cursor back a byte.
  pub fn _unget_u8(&mut self) {
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

  /// @fn unget_u32(&mut self)
  /// @brief Move the cursor back a word.
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
      addr |= self.get_u32()? as usize;
    }

    for _ in 0..size_cells {
      size <<= FDT_WORD_BYTES;
      size |= self.get_u32()? as usize;
    }

    Some((addr, size))
  }
}

/// @struct DtbHeader
/// @brief  DTB blob header information.
pub struct DtbHeader {
  dt_struct_offset: u32,
  dt_strings_offset: u32,
  _mem_rsv_map_offset: u32,
  _version: u32,
  _last_comp_version: u32,
  _boot_cpuid_phys: u32,
  _dt_strings_size: u32,
  _dt_struct_size: u32,
}

impl DtbHeader {
  fn new(cursor: &mut DtbCursor) -> Option<Self> {
    Some(DtbHeader {
      dt_struct_offset: cursor.get_u32()?,
      dt_strings_offset: cursor.get_u32()?,
      _mem_rsv_map_offset: cursor.get_u32()?,
      _version: cursor.get_u32()?,
      _last_comp_version: cursor.get_u32()?,
      _boot_cpuid_phys: cursor.get_u32()?,
      _dt_strings_size: cursor.get_u32()?,
      _dt_struct_size: cursor.get_u32()?,
    })
  }
}

/// @struct DtbRoot
/// @brief  Address and size cell lengths.
pub struct DtbRoot {
  pub addr_cells: u32,
  pub size_cells: u32,
}

/// @struct DtbPropHeader
/// @brief  Node property size and name string table offset.
pub struct DtbPropHeader {
  pub prop_size: u32,
  pub name_offset: u32,
}

/// @trait DtbScanner
/// @brief DTB scanner trait.
pub trait DtbScanner {
  /// @fn scan_node(
  ///       &mut self,
  ///       hdr: &DtbHeader,
  ///       root: &DtbRoot,
  ///       node_name: &[u8],
  ///       cursor: &mut DtbCursor,
  ///     ) -> Result<bool, DtbError>
  /// @brief   Scans the current node.
  /// @details @a scan_dtb provides the implementation with the DTB header and
  ///          root object as well as a cursor positioned at the first property
  ///          of the node. The implementation should NOT move to a position
  ///          before the current position at the start of the call. The
  ///          implementation should call @a move_to_next_property to navigate
  ///          forward and return when @a move_to_next_property returns None and
  ///          move beyond the last property in the node. The exception to this
  ///          rule is that the implemenation may use @a get_string_from_table.
  /// @param[in] hdr       The DTB header.
  /// @param[in] root      The DTB root node.
  /// @param[in] node_name The node name slice.
  /// @param[in] cursor    The DTB cursor.
  /// @returns Ok(true) if scanning should continue, Ok(false) if scanning
  ///          stop, or Err if an error is encountered.
  fn scan_node(
    &mut self,
    hdr: &DtbHeader,
    root: &DtbRoot,
    node_name: &[u8],
    cursor: &mut DtbCursor,
  ) -> Result<bool, DtbError>;
}

/// @fn scan_dtb(blob: usize, scanner: &mut impl DtbScanner) -> Result<(), DtbError>
/// @brief   Scans a DTB using a caller-defined scanner object.
/// @param[in] blob    The DTB blob to scan.
/// @param[in] scanner A scanner object.
/// @returns Ok with the total size or a DtbError.
pub fn scan_dtb(blob: usize, scanner: &mut impl DtbScanner) -> Result<u32, DtbError> {
  let total_size = check_dtb(blob)?;

  let mut cursor = DtbCursor::new(blob as *const u8, total_size);
  cursor.set_loc(FDT_WORD_BYTES * 2); // Skip magic and total size.

  let hdr = DtbHeader::new(&mut cursor).ok_or(DtbError::InvalidDtb)?;
  cursor.set_loc(hdr.dt_struct_offset); // Skip to the root node.

  let root = scan_root_node(&hdr, &mut cursor)?;

  loop {
    let begin = cursor.get_u32().ok_or(DtbError::InvalidDtb)?;

    match begin {
      FDT_BEGIN_NODE => {}
      FDT_END_NODE | FDT_NOOP => continue,
      FDT_END => break,
      _ => return Err(DtbError::InvalidDtb),
    }

    let node_name = cursor
      .get_u8_slice_null_terminated()
      .ok_or(DtbError::InvalidDtb)?;
    cursor.skip_and_align(1);

    if !scanner.scan_node(&hdr, &root, node_name, &mut cursor)? {
      break;
    }
  }

  Ok(total_size)
}

/// @fn scan_root_node(hdr: &DtbHeader, cursor: &mut DtbCursor) -> Option<DtbRoot, DtbError>
/// @brief   Scans the root node for required information.
/// @param[in] hdr    The DTB header.
/// @param[in] cursor The DTB cursor.
/// @returns The root node information or None if invalid.
fn scan_root_node(hdr: &DtbHeader, cursor: &mut DtbCursor) -> Result<DtbRoot, DtbError> {
  let begin = cursor.get_u32().ok_or(DtbError::InvalidDtb)?;

  if begin != FDT_BEGIN_NODE {
    return Err(DtbError::InvalidDtb);
  }

  let name = cursor
    .get_u8_slice_null_terminated()
    .ok_or(DtbError::InvalidDtb)?;

  if name.len() != 0 {
    return Err(DtbError::InvalidDtb);
  }

  cursor.skip_and_align(1);

  let mut root = DtbRoot {
    addr_cells: u32::MAX,
    size_cells: u32::MAX,
  };

  loop {
    let prop_hdr = match move_to_next_property(cursor) {
      Some(prop_hdr) => prop_hdr,
      _ => break,
    };

    let prop_name =
      get_string_from_table(hdr, prop_hdr.name_offset, cursor).ok_or(DtbError::InvalidDtb)?;

    if "#address-cells".as_bytes().cmp(prop_name) == cmp::Ordering::Equal {
      root.addr_cells = cursor.get_u32().ok_or(DtbError::InvalidDtb)?;
    } else if "#size-cells".as_bytes().cmp(prop_name) == cmp::Ordering::Equal {
      root.size_cells = cursor.get_u32().ok_or(DtbError::InvalidDtb)?;
    } else {
      cursor.skip_and_align(prop_hdr.prop_size);
    }
  }

  Ok(root)
}

/// @fn move_to_next_property(hdr: &DtbHeader, cursor: &mut DtbCursor) -> Option<DtbPropHeader>
/// @brief   Moves the cursor to the next property.
/// @param[in] cursor The DTB cursor.
/// @returns The property header or None if a property is not found.
pub fn move_to_next_property(cursor: &mut DtbCursor) -> Option<DtbPropHeader> {
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
      // Noops are allowed as necessary and can be skipped.
      FDT_NOOP => continue,
      // All other cases, just break.
      _ => break,
    }

    return Some(DtbPropHeader {
      prop_size: cursor.get_u32()?,
      name_offset: cursor.get_u32()?,
    });
  }

  None
}

/// @fn get_string_from_table<'cursor>(
///       hdr: &DtbHeader,
///       str_offset: u32,
///       cursor: &'cursor mut DtbCursor,
///     ) -> Option<&'cursor [u8]>
/// @brief   Retrieves a slice from the string table given an offset.
/// @param[in] hdr        The DTB header.
/// @param[in] str_offset The absolute offset of the string.
/// @param[in] cursor     The DTB cursor.
/// @returns The string as a slice or None if the string is invalid.
pub fn get_string_from_table<'cursor>(
  hdr: &DtbHeader,
  str_offset: u32,
  cursor: &'cursor mut DtbCursor,
) -> Option<&'cursor [u8]> {
  let old_loc = cursor.cur_loc;
  cursor.set_loc(hdr.dt_strings_offset + str_offset);

  let ret = cursor.get_u8_slice_null_terminated()?;
  cursor.set_loc(old_loc);

  Some(ret)
}

/// @fn get_reg_pair_size(root: &DtbRoot) -> u32
/// @brief   Calculate the expected size of a single address / size pair in a
///          reg property.
/// @details A reg property is a list of address and size pairs. Each address
///          and size is a number of u32 "cells", e.g. a 64-bit address is two
///          u32 cells. The number of cells is restricted, so the result is
///          guaranteed to fit in a u32.
/// @returns The expected size if valid.
pub fn get_reg_pair_size(root: &DtbRoot) -> Option<u32> {
  if !FDT_CELL_COUNTS.contains(&root.addr_cells) || !FDT_CELL_COUNTS.contains(&root.size_cells) {
    return None;
  }

  Some((root.addr_cells * FDT_WORD_BYTES) + (root.size_cells * FDT_WORD_BYTES))
}
