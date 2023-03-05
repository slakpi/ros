use crate::dbg_print;
use crate::support::{atags, dtb};
use core::cmp;

const MEM_RANGES: usize = 64;

/// @struct MemoryRange
/// @brief  Represents a range of memory available to the system.
#[derive(Copy, Clone)]
pub struct MemoryRange {
  pub base: usize,
  pub size: usize,
}

/// @struct MemoryConfig
/// @brief  Stores the ranges of memory available to the system and the memory
///         page size.
#[derive(Copy, Clone)]
pub struct MemoryConfig {
  ranges: [MemoryRange; MEM_RANGES],
  range_count: usize,
}

impl MemoryConfig {
  /// @fn MemoryConfig::new
  /// @brief   Construct a new MemoryConfig.
  /// @returns An empty MemoryConfig.
  pub fn new() -> Self {
    MemoryConfig {
      ranges: [MemoryRange { base: 0, size: 0 }; MEM_RANGES],
      range_count: 0,
    }
  }

  /// @fn MemoryConfig::get_ranges
  /// @brief   Access the configured memory ranges.
  /// @returns A slice with the configured memory ranges.
  pub fn get_ranges(&self) -> &[MemoryRange] {
    &self.ranges[0..self.range_count]
  }

  /// @fn MemoryConfig::insert_range
  /// @brief Insert a new memory range in order sorted by base.
  /// @param[in] range  The range to add.
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

  /// @fn MemoryConfig::trim_ranges
  /// @brief Combines ranges as necessary to ensure ranges do not overlap and
  ///        removes any empty ranges.
  /// @param[in] config The current memory configuration.
  pub fn trim_ranges(&mut self) {
    self.trim_empty_ranges();
    self.trim_overlapping_ranges();
  }

  /// @fn MemoryConfig::trim_empty_ranges
  /// @brief Removes empty ranges from the configured ranges.
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

  /// @fn MemoryConfig::trim_overlapping_ranges
  /// @brief Removes overlapping ranges from the configured ranges.
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
        // This range overlaps the next, union the ranges.
        self.ranges[i].size = b_end - a.base;
        self.ranges.copy_within((i + 2)..self.range_count, i + 1);
      } else {
        i += 1;
      }
    }
  }
}

/// @struct DtbMemoryScanner
/// @brief  Scans for DTB memory nodes. See @a dtb::DtbScanner.
struct DtbMemoryScanner<'mem> {
  config: &'mem mut MemoryConfig,
}

impl<'mem> DtbMemoryScanner<'mem> {
  /// @fn DtbMemoryScanner::check_device_type
  /// @brief   Wrapper for @a check_device_type_internal.
  /// @param[in] loc    The location of the property data in the node.
  /// @param[in] size   The size of the property.
  /// @param[in] cursor The DTB cursor.
  /// @returns See @a check_device_type_internal.
  fn check_device_type(
    loc: u32,
    size: u32,
    cursor: &mut dtb::DtbCursor,
  ) -> Result<bool, dtb::DtbError> {
    let old_loc = cursor.get_loc();

    cursor.set_loc(loc);
    let ret = DtbMemoryScanner::check_device_type_internal(size, cursor);
    cursor.set_loc(old_loc);

    ret
  }

  /// @fn DtbMemoryScanner::check_device_type_internal
  /// @brief   Check if this node describes a memory device.
  /// @pre     The cursor has been positioned at the property.
  /// @param[in] size   The size of the property.
  /// @param[in] cursor The DTB cursor.
  /// @returns Ok(true) if this is a memory device, Ok(false) if it is not, or
  ///          Err if an error is encountered.
  fn check_device_type_internal(
    size: u32,
    cursor: &mut dtb::DtbCursor,
  ) -> Result<bool, dtb::DtbError> {
    if size == 0 {
      return Err(dtb::DtbError::InvalidDtb);
    }

    let dev_type = cursor
      .get_u8_slice(size - 1)
      .ok_or(dtb::DtbError::InvalidDtb)?;

    Ok("memory".as_bytes().cmp(dev_type) == cmp::Ordering::Equal)
  }

  /// @fn DtbMemoryScanner::read_reg
  /// @brief   Wrapper for @a read_reg_internal.
  /// @param[in] loc    The location of the property data in the node.
  /// @param[in] size   The size of the property data.
  /// @param[in] root   The root node describing reg property layout.
  /// @param[in] cursor The DTB cursor.
  /// @returns See @a read_reg_internal.
  fn read_reg(
    &mut self,
    loc: u32,
    size: u32,
    root: &dtb::DtbRoot,
    cursor: &mut dtb::DtbCursor,
  ) -> Result<(), dtb::DtbError> {
    let old_loc = cursor.get_loc();

    cursor.set_loc(loc);
    let ret = self.read_reg_internal(size, root, cursor);
    cursor.set_loc(old_loc);

    ret
  }

  /// @fn DtbMemoryScanner::read_reg_internal
  /// @brief   Read a reg property.
  /// @pre     The cursor has been positioned at the property.
  /// @param[in] size   The size of the property data.
  /// @param[in] root   The root node describing reg property layout.
  /// @param[in] cursor The DTB cursor.
  /// @returns Ok if the reg property is valid or Err if an error is
  ///          encountered.
  fn read_reg_internal(
    &mut self,
    size: u32,
    root: &dtb::DtbRoot,
    cursor: &mut dtb::DtbCursor,
  ) -> Result<(), dtb::DtbError> {
    let reg_size = dtb::get_reg_pair_size(root).ok_or(dtb::DtbError::InvalidDtb)?;

    // Check that the size non-zero and a multiple of the size specified by the
    // root node.
    if reg_size == 0 || size % reg_size != 0 {
      return Err(dtb::DtbError::InvalidDtb);
    }

    let reg_count = cmp::min((size / reg_size) as usize, MEM_RANGES);

    for _ in 0..reg_count {
      let (base, size) = cursor
        .get_reg(root.addr_cells, root.size_cells)
        .ok_or(dtb::DtbError::InvalidDtb)?;
      self.config.insert_range(MemoryRange { base, size });
    }

    Ok(())
  }
}

impl<'mem> dtb::DtbScanner for DtbMemoryScanner<'mem> {
  /// @fn DtbMemoryScanner::scan_node
  /// @brief See @a dtb::DtbScanner.
  fn scan_node(
    &mut self,
    hdr: &dtb::DtbHeader,
    root: &dtb::DtbRoot,
    _node_name: &[u8],
    cursor: &mut dtb::DtbCursor,
  ) -> Result<bool, dtb::DtbError> {
    let mut dev_type = (0u32, 0, false);
    let mut reg = (0u32, 0, false);

    while let Some(prop_hdr) = dtb::move_to_next_property(cursor) {
      let prop_name = dtb::get_string_from_table(hdr, prop_hdr.name_offset, cursor)
        .ok_or(dtb::DtbError::InvalidDtb)?;

      if "device_type".as_bytes().cmp(prop_name) == cmp::Ordering::Equal {
        dev_type = (cursor.get_loc(), prop_hdr.prop_size, true);
      } else if "reg".as_bytes().cmp(prop_name) == cmp::Ordering::Equal {
        reg = (cursor.get_loc(), prop_hdr.prop_size, true);
      }

      cursor.skip_and_align(prop_hdr.prop_size);
    }

    // If the node did not contain device_type or reg, keep scanning.
    if !dev_type.2 || !reg.2 {
      return Ok(true);
    }

    // If the node is not a memory device, keep scanning.
    if !DtbMemoryScanner::check_device_type(dev_type.0, dev_type.1, cursor)? {
      return Ok(true);
    }

    self.read_reg(reg.0, reg.1, root, cursor)?;

    // Keep scanning if we have not filled the memory ranges yet.
    Ok(self.config.range_count < MEM_RANGES)
  }
}

/// @struct AtagMemoryScanner
/// @brief  Scans for MEM tags. See @a atags::AtagScanner.
struct AtagMemoryScanner<'mem> {
  config: &'mem mut MemoryConfig,
}

impl<'mem> atags::AtagScanner for AtagMemoryScanner<'mem> {
  /// @fn AtagMemoryScanner::scan_mem_tag
  /// @brief See @a atags::AtagScanner.
  fn scan_mem_tag(&mut self, mem: &atags::AtagMem) -> Result<bool, atags::AtagError> {
    self.config.insert_range(MemoryRange {
      base: mem.base as usize,
      size: mem.size as usize,
    });

    if self.config.range_count == MEM_RANGES {
      return Ok(false);
    }

    Ok(true)
  }
}

/// @fn get_memory_layout
/// @brief   Get the system memory layout.
/// @param[in] blob      The DTB or ATAGs blob address.
/// @returns The memory layout or None if unable to read the DTB or ATAGs.
pub fn get_memory_layout(blob: usize) -> Option<MemoryConfig> {
  let mut config = MemoryConfig::new();

  let ok = match get_memory_layout_from_dtb(blob, &mut config) {
    // Successfully read the DTB memory configuration.
    Ok(_) => true,
    // The blob does not contain a DTB, try ATAGs.
    Err(dtb::DtbError::NotADtb) => get_memory_layout_from_atags(blob, &mut config).is_ok(),
    // The DTB was invalid, fail out.
    Err(_) => false,
  };

  if !ok {
    dbg_print!("Memory: Could not read a valid device tree or ATAG list.");
    return None;
  }

  config.trim_ranges();

  if config.range_count == 0 {
    dbg_print!("Memory: No valid memory ranges available.");
    return None;
  }

  Some(config)
}

/// @fn get_memory_layout_from_dtb
/// @brief   Get the system memory layout from a DTB.
/// @param[in] blob The DTB blob.
/// @returns The scan result.
fn get_memory_layout_from_dtb(
  blob: usize,
  config: &mut MemoryConfig,
) -> Result<u32, dtb::DtbError> {
  let mut scanner = DtbMemoryScanner { config };
  dtb::scan_dtb(blob, &mut scanner)
}

/// @fn get_memory_layout_from_atags
/// @brief Get the system memory layout from ATAGs.
/// @param[in] blob The ATAGs blob.
fn get_memory_layout_from_atags(
  blob: usize,
  config: &mut MemoryConfig,
) -> Result<(), atags::AtagError> {
  let mut scanner = AtagMemoryScanner { config };
  atags::scan_atags(blob, &mut scanner)
}
