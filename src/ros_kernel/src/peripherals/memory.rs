//! DTB physical memory scanning.

use crate::support::dtb;
use core::cmp;

const MEM_RANGES: usize = 64;

/// Represents a range of memory available to the system.
#[derive(Copy, Clone)]
pub struct MemoryRange {
  pub base: usize,
  pub size: usize,
}

/// Stores the ranges of memory available to the system.
#[derive(Copy, Clone)]
pub struct MemoryConfig {
  ranges: [MemoryRange; MEM_RANGES],
  range_count: usize,
}

impl MemoryConfig {
  /// Construct a new MemoryConfig.
  ///
  /// # Returns
  ///
  /// An empty MemoryConfig.
  pub fn new() -> Self {
    MemoryConfig {
      ranges: [MemoryRange {
        base: 0,
        size: 0,
      }; MEM_RANGES],
      range_count: 0,
    }
  }

  /// Access the configured memory ranges.
  ///
  /// # Returns
  ///
  /// A slice with the valid memory ranges stored in the configuration.
  pub fn get_ranges(&self) -> &[MemoryRange] {
    &self.ranges[0..self.range_count]
  }

  /// Insert a new memory range in order sorted by base.
  ///
  /// # Parameters
  ///
  /// * `range` - The new block of memory to add to the configuration.
  pub fn insert_range(&mut self, range: MemoryRange) {
    if self.range_count >= MEM_RANGES {
      return;
    }

    let mut ins = self.range_count;

    for i in 0..self.range_count {
      if range.base <= self.ranges[i].base {
        ins = i;
        break;
      }
    }

    self.ranges.copy_within(ins..self.range_count, ins + 1);
    self.range_count += 1;
    self.ranges[ins] = range;
  }

  /// Combines ranges as necessary to ensure ranges do not overlap and removes
  /// any empty ranges.
  pub fn trim_ranges(&mut self) {
    self.trim_overlapping_ranges();
    self.trim_empty_ranges();
  }

  /// Removes empty ranges from the configured ranges.
  fn trim_empty_ranges(&mut self) {
    let mut i = 0usize;

    while i < self.range_count {
      if self.ranges[i].size > 0 {
        i += 1;
        continue;
      }

      self.ranges.copy_within((i + 1)..self.range_count, i);
      self.range_count -= 1;
    }
  }

  /// Removes overlapping ranges from the configured ranges.
  fn trim_overlapping_ranges(&mut self) {
    if self.range_count < 2 {
      return;
    }

    let mut i = 0usize;

    while i < self.range_count - 1 {
      let a = &self.ranges[i];
      let b = &self.ranges[i + 1];
      let a_end = a.base + a.size;
      let b_end = b.base + b.size;

      if a.base <= b.base && a_end >= b_end {
        // This range encompasses the next range, remove the next range.
        self.ranges.copy_within((i + 2)..self.range_count, i + 1);
      } else if b.base < a.base && b_end > a_end {
        // The next range encompasses this range, remove this range.
        self.ranges.copy_within((i + 1)..self.range_count, i);
      } else if a.base <= b.base && a_end > b.base {
        // This range overlaps the next. Union the ranges and remove the
        // extraneous range.
        self.ranges[i].size = b_end - a.base;
        self.ranges.copy_within((i + 2)..self.range_count, i + 1);
      } else {
        i += 1;
      }
    }
  }
}

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

      self.config.insert_range(MemoryRange {
        base,
        size,
      });
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

  if config.range_count == 0 {
    return None;
  }

  Some(config)
}
