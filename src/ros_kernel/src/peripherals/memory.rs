//! DTB physical memory scanning.

use crate::support::{dtb, range, range_set};
use core::cmp;

/// Maximum number of memory ranges that can be stored in a configuration.
pub const MAX_MEM_RANGES: usize = 64;

pub type MemoryConfig = range_set::RangeSet<MAX_MEM_RANGES>;

/// Scans for DTB memory nodes.
struct DtbMemoryScanner<'mem> {
  config: &'mem mut MemoryConfig,
  addr_cells: u32,
  size_cells: u32,
}

impl<'mem> DtbMemoryScanner<'mem> {
  /// Construct a new DTB memory scanner.
  ///
  /// # Parameters
  ///
  /// * `config` - The MemoryConfig that will store the ranges found in the DTB.
  ///
  /// # Returns
  ///
  /// A new DtbMemoryScanner.
  pub fn new(config: &'mem mut MemoryConfig) -> Self {
    DtbMemoryScanner {
      config,
      addr_cells: 0,
      size_cells: 0,
    }
  }

  /// Reads the root cell configuration.
  ///
  /// # Parameters
  ///
  /// * `reader` - The DTB reader.
  /// * `cursor` - The cursor pointing to the root node.
  ///
  /// # Returns
  ///
  /// Returns Ok if able to read the cell configuration, otherwise a DTB error.
  fn scan_root_node(
    &mut self,
    reader: &dtb::DtbReader,
    cursor: &dtb::DtbCursor,
  ) -> Result<(), dtb::DtbError> {
    let mut tmp_cursor = *cursor;

    while let Some(header) = reader.get_next_property(&mut tmp_cursor) {
      let name = reader
        .get_slice_from_string_table(header.name_offset)
        .ok_or(dtb::DtbError::InvalidDtb)?;

      if "#address-cells".as_bytes().cmp(name) == cmp::Ordering::Equal {
        self.addr_cells = reader
          .get_u32(&mut tmp_cursor)
          .ok_or(dtb::DtbError::InvalidDtb)?;
      } else if "#size-cells".as_bytes().cmp(name) == cmp::Ordering::Equal {
        self.size_cells = reader
          .get_u32(&mut tmp_cursor)
          .ok_or(dtb::DtbError::InvalidDtb)?;
      } else {
        reader.skip_and_align(header.size, &mut tmp_cursor);
      }
    }

    Ok(())
  }

  /// Scans a device node. If the device is a memory device, the function adds
  /// the memory ranges to the memory layout.
  ///
  /// # Parameters
  ///
  /// * `reader` - The DTB reader.
  /// * `cursor` - The cursor pointing to the root node.
  ///
  /// # Returns
  ///
  /// Returns Ok if able to read the device node, otherwise a DTB error.
  fn scan_device_node(
    &mut self,
    reader: &dtb::DtbReader,
    cursor: &dtb::DtbCursor,
  ) -> Result<bool, dtb::DtbError> {
    let mut tmp_cursor = *cursor;
    let mut dev_type = (tmp_cursor, 0usize, false);
    let mut reg = (tmp_cursor, 0usize, false);
    let mut addr_cells = self.addr_cells;
    let mut size_cells = self.size_cells;

    while let Some(header) = reader.get_next_property(&mut tmp_cursor) {
      let name = reader
        .get_slice_from_string_table(header.name_offset)
        .ok_or(dtb::DtbError::InvalidDtb)?;

      if "device_type".as_bytes().cmp(name) == cmp::Ordering::Equal {
        dev_type = (tmp_cursor, header.size, true);
      } else if "reg".as_bytes().cmp(name) == cmp::Ordering::Equal {
        reg = (tmp_cursor, header.size, true);
      } else if "#address-cells".as_bytes().cmp(name) == cmp::Ordering::Equal {
        addr_cells = reader
          .get_u32(&mut tmp_cursor)
          .ok_or(dtb::DtbError::InvalidDtb)?;
        continue;
      } else if "#size-cells".as_bytes().cmp(name) == cmp::Ordering::Equal {
        size_cells = reader
          .get_u32(&mut tmp_cursor)
          .ok_or(dtb::DtbError::InvalidDtb)?;
        continue;
      }

      reader.skip_and_align(header.size, &mut tmp_cursor);
    }

    if !dev_type.2 || !self.check_device_type(dev_type.1, reader, &dev_type.0) {
      return Ok(true);
    }

    if !reg.2 {
      return Ok(true);
    }

    self.read_memory_reg(reg.1, addr_cells, size_cells, reader, &reg.0)
  }

  fn check_device_type(
    &self,
    _prop_size: usize,
    reader: &dtb::DtbReader,
    cursor: &dtb::DtbCursor,
  ) -> bool {
    let mut tmp_cursor = *cursor;

    if let Some(name) = reader.get_null_terminated_u8_slice(&mut tmp_cursor) {
      return "memory".as_bytes().cmp(name) == cmp::Ordering::Equal;
    }

    false
  }

  fn read_memory_reg(
    &mut self,
    prop_size: usize,
    addr_cells: u32,
    size_cells: u32,
    reader: &dtb::DtbReader,
    cursor: &dtb::DtbCursor,
  ) -> Result<bool, dtb::DtbError> {
    let reg_size = dtb::DtbReader::get_reg_size(addr_cells, size_cells);
    let mut tmp_cursor = *cursor;

    if (reg_size == 0) || (prop_size == 0) || (prop_size < reg_size) || (prop_size % reg_size != 0)
    {
      return Err(dtb::DtbError::InvalidDtb);
    }

    for _ in 0..(prop_size / reg_size) {
      let (base, size) = reader
        .get_reg(addr_cells, size_cells, &mut tmp_cursor)
        .ok_or(dtb::DtbError::InvalidDtb)?;

      self.config.insert_range(range::Range { base, size });
    }

    Ok(true)
  }
}

impl<'mem> dtb::DtbScanner for DtbMemoryScanner<'mem> {
  fn scan_node(
    &mut self,
    reader: &dtb::DtbReader,
    name: &[u8],
    cursor: &dtb::DtbCursor,
  ) -> Result<bool, dtb::DtbError> {
    if name.len() == 0 {
      _ = self.scan_root_node(reader, cursor)?;
      return Ok(true);
    }

    self.scan_device_node(reader, cursor)
  }
}

/// Get the system memory layout.
pub fn get_memory_layout(blob: usize) -> Option<MemoryConfig> {
  let mut config = MemoryConfig::new();
  let mut scanner = DtbMemoryScanner::new(&mut config);
  let reader = dtb::DtbReader::new(blob).ok()?;

  _ = reader.scan(&mut scanner).ok()?;

  config.trim_ranges();

  if config.is_empty() {
    return None;
  }

  Some(config)
}
