//! ARM Memory Peripheral Utilities

use crate::support::{dtb, hash, hash_map, range, range_set};
use core::cmp;

/// Maximum number of memory ranges that can be stored in a configuration.
pub const MAX_MEM_RANGES: usize = 64;

pub type MemoryConfig = range_set::RangeSet<MAX_MEM_RANGES>;

/// Tags for expected properties and values.
enum StringTag {
  DtbPropAddressCells,
  DtbPropSizeCells,
  DtbPropDeviceType,
  DtbPropReg,
  DtbValueMemory,
}
type StringMap<'map> = hash_map::HashMap<&'map [u8], StringTag, hash::BuildFnv1aHasher, 31>;

/// Scans for DTB memory nodes.
struct DtbMemoryScanner<'mem> {
  config: &'mem mut MemoryConfig,
  string_map: StringMap<'mem>,
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
      string_map: Self::build_string_map(),
      addr_cells: 0,
      size_cells: 0,
    }
  }

  /// Build a string map for the scanner.
  ///
  /// # Returns
  ///
  /// A new string map for the expected properties and values.
  fn build_string_map() -> StringMap<'mem> {
    let mut string_map = StringMap::with_hasher_factory(hash::BuildFnv1aHasher {});
    string_map.insert("#address-cells".as_bytes(), StringTag::DtbPropAddressCells);
    string_map.insert("#size-cells".as_bytes(), StringTag::DtbPropSizeCells);
    string_map.insert("device_type".as_bytes(), StringTag::DtbPropDeviceType);
    string_map.insert("reg".as_bytes(), StringTag::DtbPropReg);
    string_map.insert("memory".as_bytes(), StringTag::DtbValueMemory);
    string_map
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
      let tag = self.string_map.find(header.name);

      match tag {
        Some(StringTag::DtbPropAddressCells) => {
          self.addr_cells = Self::read_addr_cells(reader, &mut tmp_cursor)?;
        }
        Some(StringTag::DtbPropSizeCells) => {
          self.size_cells = Self::read_size_cells(reader, &mut tmp_cursor)?;
        }
        _ => reader.skip_and_align(header.size, &mut tmp_cursor),
      }
    }

    Ok(())
  }

  /// Read the `#address-cells` property.
  ///
  /// # Parameters
  ///
  /// * `reader` - The DTB reader.
  /// * `cursor` - The current position in the DTB.
  ///
  /// # Returns
  ///
  /// Returns Ok with the number of address cells if valid, otherwise a DTB
  /// error.
  fn read_addr_cells(
    reader: &dtb::DtbReader,
    cursor: &mut dtb::DtbCursor,
  ) -> Result<u32, dtb::DtbError> {
    let addr_cells = reader.get_u32(cursor).ok_or(dtb::DtbError::InvalidDtb)?;
    Ok(addr_cells)
  }

  /// Read the `#size-cells` property.
  ///
  /// # Parameters
  ///
  /// * `reader` - The DTB reader.
  /// * `cursor` - The current position in the DTB.
  ///
  /// # Returns
  ///
  /// Returns Ok with the number of size cells if valid, otherwise a DTB error.
  fn read_size_cells(
    reader: &dtb::DtbReader,
    cursor: &mut dtb::DtbCursor,
  ) -> Result<u32, dtb::DtbError> {
    let size_cells = reader.get_u32(cursor).ok_or(dtb::DtbError::InvalidDtb)?;
    Ok(size_cells)
  }

  /// Scans a device node. If the device is a memory device, the function adds
  /// the memory ranges to the memory layout.
  ///
  /// # Parameters
  ///
  /// * `reader` - The DTB reader.
  /// * `cursor` - The cursor pointing to the device node.
  ///
  /// # Returns
  ///
  /// Returns Ok if able to read the device node, otherwise a DTB error.
  fn scan_device_node(
    &mut self,
    reader: &dtb::DtbReader,
    cursor: &dtb::DtbCursor,
  ) -> Result<(), dtb::DtbError> {
    let mut tmp_cursor = *cursor;
    let mut dev_type = (tmp_cursor, 0usize, false);
    let mut reg = (tmp_cursor, 0usize, false);
    let mut addr_cells = self.addr_cells;
    let mut size_cells = self.size_cells;

    while let Some(header) = reader.get_next_property(&mut tmp_cursor) {
      let tag = self.string_map.find(header.name);

      match tag {
        Some(StringTag::DtbPropDeviceType) => dev_type = (tmp_cursor, header.size, true),
        Some(StringTag::DtbPropReg) => reg = (tmp_cursor, header.size, true),
        Some(StringTag::DtbPropAddressCells) => {
          addr_cells = Self::read_addr_cells(reader, &mut tmp_cursor)?;
          continue;
        }
        Some(StringTag::DtbPropSizeCells) => {
          size_cells = Self::read_size_cells(reader, &mut tmp_cursor)?;
          continue;
        }
        _ => {}
      }

      reader.skip_and_align(header.size, &mut tmp_cursor);
    }

    if !dev_type.2 || !self.check_device_type(dev_type.1, reader, &dev_type.0) {
      return Ok(());
    }

    if !reg.2 {
      return Ok(());
    }

    self.add_memory_blocks(reg.1, addr_cells, size_cells, reader, &reg.0)
  }

  /// Check for a memory device.
  ///
  /// # Parameters
  ///
  /// * `prop_size` - The size of the property value.
  /// * `reader` - The DTB reader.
  /// * `cursor` - The current position in the DTB.
  ///
  /// # Returns
  ///
  /// Returns true if the device is a memory device, false otherwise.
  fn check_device_type(
    &self,
    _prop_size: usize,
    reader: &dtb::DtbReader,
    cursor: &dtb::DtbCursor,
  ) -> bool {
    let mut tmp_cursor = *cursor;

    if let Some(name) = reader.get_null_terminated_u8_slice(&mut tmp_cursor) {
      match self.string_map.find(name) {
        Some(StringTag::DtbValueMemory) => return true,
        _ => {}
      }
    }

    false
  }

  /// Read a memory register property of (base address, size) pairs and add them
  /// to the memory configuration.
  ///
  /// # Parameters
  ///
  /// * `prop_size` - The size of the register property.
  /// * `addr_cells` - The number of address cells.
  /// * `size_cells` - The number of size cells.
  /// * `reader` - The DTB reader.
  /// * `cursor` - The current position in the DTB.
  ///
  /// # Returns
  ///
  /// Returns Ok if able to read the register property, otherwise a DTB error.
  fn add_memory_blocks(
    &mut self,
    prop_size: usize,
    addr_cells: u32,
    size_cells: u32,
    reader: &dtb::DtbReader,
    cursor: &dtb::DtbCursor,
  ) -> Result<(), dtb::DtbError> {
    let reg_size = dtb::DtbReader::get_reg_size(addr_cells, size_cells);
    let mut tmp_cursor = *cursor;

    // Sanity check the DTB.
    if (reg_size == 0) || (prop_size == 0) || (prop_size < reg_size) || (prop_size % reg_size != 0)
    {
      return Err(dtb::DtbError::InvalidDtb);
    }

    for _ in 0..(prop_size / reg_size) {
      let (base, size) = reader
        .get_reg(addr_cells, size_cells, &mut tmp_cursor)
        .ok_or(dtb::DtbError::InvalidDtb)?;

      // The base is beyond the platform's addressable range, just skip it.
      if base > usize::MAX as u64 {
        continue;
      }

      // The base address is known to be less than the maximum platform address.
      // Subtract the base to get the maximum allowable size and clamp the size
      // from the DTB.
      let max_size = (usize::MAX as u64) - base;
      let size = cmp::min(size, max_size);
      _ = self.config.insert_range(range::Range {
        base: base as usize,
        size: size as usize,
      });
    }

    Ok(())
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
    } else {
      _ = self.scan_device_node(reader, cursor)?;
    }

    Ok(true)
  }
}

/// Get the system memory layout.
///
/// # Parameters
///
/// * `config` - The memory configuration.
/// * `blob` - The DTB address.
///
/// # Assumptions
///
/// Assumes the configuration is empty.
///
/// # Returns
///
/// True if able to read the memory configuration and at least one valid memory
/// range is provided by the SoC, false otherwise.
pub fn get_memory_layout(config: &mut MemoryConfig, blob: usize) -> bool {
  debug_assert!(config.is_empty());

  let mut scanner = DtbMemoryScanner::new(config);

  let reader = match dtb::DtbReader::new(blob) {
    Ok(r) => r,
    _ => return false,
  };

  if !reader.scan(&mut scanner).is_ok() {
    return false;
  }

  config.trim_ranges();

  if config.is_empty() {
    return false;
  }

  true
}
