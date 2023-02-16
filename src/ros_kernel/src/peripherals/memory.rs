use crate::dbg_print;
use crate::support::{atags, bits, dtb};
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

  /// @fn MemoryConfig::exclude_range
  /// @brief Excludes a memory range from the configured ranges.
  /// @param[in] excl      The exclusion range. Does not need to be page aligned.
  /// @param[in] page_size The memory page size for alignment.
  pub fn exclude_range(&mut self, excl: &MemoryRange, page_size: usize) {
    if excl.size == 0 {
      return;
    }

    self.trim_ranges();

    let mut i = 0usize;

    while i < self.range_count {
      let split = Self::split_range(&self.ranges[i], excl, page_size);

      // If the first element is valid, the current range can simply be
      // replaced.
      if let Some(a) = split.0 {
        self.ranges[i] = a;
      }

      if let Some(b) = split.1 {
        if split.0.is_none() {
          // Just replace the current range.
          self.ranges[i] = b;
        } else if self.range_count < MEM_RANGES {
          // Insert the new range after the current range. Increment the index
          // an extra time.
          self.ranges.copy_within(i..self.range_count, i + 1);
          self.range_count += 1;
          self.ranges[i + 1] = b;
          i += 1;
        } else {
          // TODO: Either we hit a bug or we're not accounting for
          // configurations that create a bunch of memory ranges. Either way,
          // there is probably a more graceful way to handle this.
          panic!("Unable to exclude memory range.");
        }
      }

      // If neither element is valid, remove the current range. Do not increment
      // the index yet.
      if split.0.is_none() && split.1.is_none() {
        self.ranges.copy_within((i + 1)..self.range_count, i);
        self.range_count -= 1;
        continue;
      }

      i += 1;
    }
  }

  /// @fn MemoryConfig::trim_ranges
  /// @brief Removes overlapping or null ranges from the configured ranges.
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

  /// @fn MemoryRange::split_range
  /// @brief   Splits a range using an exclusion range.
  /// @returns A tuple handling the following cases:
  ///
  ///          * If the ranges are mutually exclusive, returns the original
  ///            range as the first element in the tuple and None for the
  ///            second.
  ///
  ///          * If the exclusion range fully encompasses the range, returns
  ///            None for both elements of the tuple.
  ///
  ///          * If the down page-aligned base, EE, of the exclusion range is
  ///            greater than the range base, returns a new range in the first
  ///            element of the tuple with the original base and a new size
  ///            calculated using EE as the end. Otherwise, returns None in the
  ///            first element of the tuple.
  ///
  ///            If the up page-aligned end, EB, of the exclusion range is less
  ///            than the range end, returns a new range in the second element
  ///            of the tuple with EB as the base a new size calculated using
  ///            the original end. Otherwise, returns None in the second element
  ///            of the tuple.
  ///
  ///          The last case handles the exclusion range being fully encompassed
  ///          by the range as well as the exclusion range overlapping either
  ///          end of the range and handles returning None if the overlap
  ///          results in empty ranges.
  fn split_range(
    range: &MemoryRange,
    excl: &MemoryRange,
    page_size: usize,
  ) -> (Option<MemoryRange>, Option<MemoryRange>) {
    let range_end = range.base + range.size;
    let excl_end = excl.base + excl.size;

    if excl_end < range.base || range_end < excl.base {
      return (Some(*range), None);
    }

    if excl.base <= range.base && excl_end >= range_end {
      return (None, None);
    }

    let end = bits::align_down(excl.base, page_size);
    let base = bits::align_up(excl_end, page_size);

    let a = if end > range.base {
      Some(MemoryRange {
        base: range.base,
        size: end - range.base,
      })
    } else {
      None
    };

    let b = if base < range_end {
      Some(MemoryRange {
        base,
        size: range_end - base,
      })
    } else {
      None
    };

    (a, b)
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
