use crate::dbg_print;
use crate::support::{atags, bits, dtb};
use core::cmp;

const MEM_RANGES: usize = 64;

/// @struct MemoryRange
/// @brief  Represents a range of memory available to the system.
#[derive(Copy, Clone)]
struct MemoryRange {
  base: usize,
  size: usize,
}

/// @struct MemoryConfig
/// @brief  Stores the ranges of memory available to the system and the memory
///         page size.
struct MemoryConfig {
  ranges: [MemoryRange; MEM_RANGES],
  range_count: u8,
  page_size: usize,
}

/// @struct DtbMemoryScanner
/// @brief  Scans for DTB memory nodes. See @a dtb::DtbScanner.
struct DtbMemoryScanner<'mem> {
  config: &'mem mut MemoryConfig,
}

impl<'mem> DtbMemoryScanner<'mem> {
  /// @fn check_device_type(
  ///       loc: u32,
  ///       size: u32,
  ///       cursor: &mut dtb::DtbCursor,
  ///     ) -> Result<bool, dtb::DtbError>
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

  /// @fn check_device_type_internal(
  ///       size: u32,
  ///       cursor: &mut dtb::DtbCursor,
  ///     ) -> Result<bool, dtb::DtbError>
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

  /// @fn read_reg(
  ///       &mut self,
  ///       loc: u32,
  ///       size: u32,
  ///       root: &dtb::DtbRoot,
  ///       cursor: &mut dtb::DtbCursor,
  ///     ) -> Result<(), dtb::DtbError>
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

  /// @fn read_reg_internal(
  ///       &mut self,
  ///       size: u32,
  ///       root: &dtb::DtbRoot,
  ///       cursor: &mut dtb::DtbCursor,
  ///     ) -> Result<(), dtb::DtbError>
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

      // Insert the new range in sorted order by range base.
      for i in 0..=self.config.range_count as usize {
        if base <= self.config.ranges[i].base {
          self
            .config
            .ranges
            .copy_within(i..self.config.range_count as usize, i + 1);
          self.config.range_count += 1;

          let range = &mut self.config.ranges[i];
          range.base = base;
          range.size = size;
          break;
        }
      }
    }

    Ok(())
  }
}

impl<'mem> dtb::DtbScanner for DtbMemoryScanner<'mem> {
  /// @fn scan_node(
  ///       &mut self,
  ///       hdr: &dtb::DtbHeader,
  ///       root: &dtb::DtbRoot,
  ///       node_name: &[u8],
  ///       cursor: &mut dtb::DtbCursor,
  ///     ) -> Result<bool, dtb::DtbError>
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

    loop {
      let prop_hdr = match dtb::move_to_next_property(cursor) {
        Some(prop_hdr) => prop_hdr,
        _ => break,
      };

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

    _ = self.read_reg(reg.0, reg.1, root, cursor)?;

    // Keep scanning if we have not filled the memory ranges yet.
    Ok((self.config.range_count as usize) < MEM_RANGES)
  }
}

/// @struct AtagMemoryScanner
/// @brief  Scans for MEM tags. See @a atags::AtagScanner.
struct AtagMemoryScanner<'mem> {
  config: &'mem mut MemoryConfig,
}

impl<'mem> atags::AtagScanner for AtagMemoryScanner<'mem> {
  /// @fn scan_mem_tag(&mut self, mem: &atags::AtagMem) -> Result<bool, atags::AtagError>
  /// @brief See @a atags::AtagScanner.
  fn scan_mem_tag(&mut self, mem: &atags::AtagMem) -> Result<bool, atags::AtagError> {
    for i in 0..=self.config.range_count as usize {
      if mem.base as usize <= self.config.ranges[i].base {
        self
          .config
          .ranges
          .copy_within(i..self.config.range_count as usize, i + 1);
        self.config.range_count += 1;

        let range = &mut self.config.ranges[i];
        range.base = mem.base as usize;
        range.size = mem.size as usize;
        break;
      }
    }

    if self.config.range_count as usize == MEM_RANGES {
      return Ok(false);
    }

    Ok(true)
  }
}

/// @var   MEMORY_CONFIG
/// @brief The system memory configuration. The kernel is single-threaded, so
///        directly accessing the value is safe.
static mut MEMORY_CONFIG: MemoryConfig = MemoryConfig {
  ranges: [MemoryRange { base: 0, size: 0 }; MEM_RANGES],
  range_count: 0,
  page_size: 0,
};

/// @fn init_memory(blob: usize)
/// @brief   Initialize the system memory configuration.
/// @param[in] blob        The DTB or ATAGs blob.
/// @param[in] page_size   The memory page size to use.
/// @param[in] kernel_base The location of the kernel in memory.
/// @param[in] kernel_size The size of the kernel image.
/// @returns True if memory is successfully initialized.
pub fn init_memory(blob: usize, page_size: usize, kernel_base: usize, kernel_size: usize) -> bool {
  let config = unsafe { &mut MEMORY_CONFIG };

  // For now, the kernel and DTB are the only holes we need to poke in the
  // configured address ranges. We exclude 0 up to the kernel size which
  // includes ATAGs (based at 0x100). This assumes the kernel is somewhere near
  // the beginning of the address range...which is an assumption that may need
  // to be checked at some point.
  let mut excl = [
    MemoryRange {
      base: 0,
      size: kernel_base + kernel_size,
    },
    MemoryRange { base: 0, size: 0 },
  ];

  debug_assert!(config.range_count == 0);

  if page_size == 0 || !bits::is_power_of_2(page_size) {
    dbg_print!("Memory: Page size is not a power of 2.\n");
    debug_assert!(false);
    return false;
  }

  let ok = match init_memory_from_dtb(blob, config) {
    // Success scanning the DTB, exclude the memory region it occupies.
    Ok(total_size) => {
      excl[1].base = blob;
      excl[1].size = total_size as usize;
      true
    }
    // The memory does not contain a DTB, try ATAGs.
    Err(dtb::DtbError::NotADtb) => init_memory_from_atags(blob, config).is_ok(),
    // The DTB was invalid, fail out.
    Err(_) => false,
  };

  if !ok {
    dbg_print!("Memory: Could not read a valid device tree or ATAG list.\n");
    debug_assert!(false);
    config.range_count = 0;
    return false;
  }

  config.page_size = page_size;

  finalize_ranges(config, &excl);

  if config.range_count == 0 {
    dbg_print!("Memory: No valid memory ranges available.\n");
    debug_assert!(false);
    return false;
  }

  true
}

/// @fn init_memory_from_dtb(blob: usize, config: &mut MemoryConfig)
/// @brief   Initialize the system memory configuration from a DTB.
/// @param[in] blob The DTB blob.
/// @returns The scan result.
fn init_memory_from_dtb(blob: usize, config: &mut MemoryConfig) -> Result<u32, dtb::DtbError> {
  let mut scanner = DtbMemoryScanner { config: config };
  dtb::scan_dtb(blob, &mut scanner)
}

/// @fn init_memory_from_atags(blob: usize, config: &mut MemoryConfig)
/// @brief Initialize the system memory configuration from ATAGs.
/// @param[in] blob The ATAGs blob.
fn init_memory_from_atags(blob: usize, config: &mut MemoryConfig) -> Result<(), atags::AtagError> {
  let mut scanner = AtagMemoryScanner { config: config };
  atags::scan_atags(blob, &mut scanner)
}

/// @fn finalize_ranges(config: &mut MemoryConfig, excl: &[MemoryRange])
/// @brief Modifies the configured ranges to exclude the specified ranges and
///        trims any empty ranges.
/// @param[in] config The memory configuration.
/// @param[in] excl   The ranges to exclude.
fn finalize_ranges(config: &mut MemoryConfig, excl: &[MemoryRange]) {
  // Trim the memory configuration before doing exclusion operations.
  trim_ranges(config);

  for e in excl {
    exclude_range(config, e);
    exclude_range(config, e);
  }

  // Re-trim to get any empty ranges left over after exclusion.
  trim_ranges(config);
}

/// @fn fn trim_ranges(config: &mut MemoryConfig)
/// @brief Removes overlapping or null ranges from the configured ranges.
/// @pre   The configured ranges must be sorted by base address.
/// @param[in] config The current memory configuration.
fn trim_ranges(config: &mut MemoryConfig) {
  trim_empty_ranges(config);
  trim_overlapping_ranges(config);
}

/// @fn fn trim_empty_ranges(config: &mut MemoryConfig)
/// @brief Removes empty ranges from the configured ranges.
/// @pre   The configured ranges must be sorted by base address.
/// @param[in] config The current memory configuration.
fn trim_empty_ranges(config: &mut MemoryConfig) {
  let mut i = 0usize;

  while i < config.range_count as usize {
    if config.ranges[i].size > 0 {
      i += 1;
      continue;
    }

    config
      .ranges
      .copy_within((i + 1)..config.range_count as usize, i);
    config.range_count -= 1;
  }
}

/// @fn fn trim_overlapping_ranges(config: &mut MemoryConfig)
/// @brief Removes overlapping ranges from the configured ranges.
/// @pre   The configured ranges must be sorted by base address.
/// @param[in] config The current memory configuration.
fn trim_overlapping_ranges(config: &mut MemoryConfig) {
  if config.range_count < 2 {
    return;
  }

  let mut i = 0usize;

  while i < (config.range_count - 1) as usize {
    let a = &config.ranges[i];
    let b = &config.ranges[i + 1];
    let a_end = a.base + a.size;
    let b_end = b.base + b.size;

    if a.base <= b.base && a_end >= b_end {
      // This range encompasses the next range, remove the next range.
      config
        .ranges
        .copy_within((i + 2)..config.range_count as usize, i + 1);
    } else if b.base < a.base && b_end > a_end {
      // The next range encompasses this range, remove this range.
      config
        .ranges
        .copy_within((i + 1)..config.range_count as usize, i);
    } else if a.base <= b.base && a_end > b.base {
      // This range overlaps the next, union the ranges.
      config.ranges[i].size = b_end - a.base;
      config
        .ranges
        .copy_within((i + 2)..config.range_count as usize, i + 1);
    } else {
      i += 1;
    }
  }
}

/// @fn exclude_range(config: &mut MemoryConfig, excl: &MemoryRange)
/// @brief Excludes a memory range from the configured ranges.
/// @pre   The configured ranges must be sorted by base address and empty ranges
///        have been trimmed.
/// @param[in] config The current memory configuration.
/// @param[in] excl   The exclusion range. Does not need to be page aligned.
fn exclude_range(config: &mut MemoryConfig, excl: &MemoryRange) {
  if excl.size == 0 {
    return;
  }

  let mut i = 0usize;

  while i < config.range_count as usize {
    let split = split_range(&config.ranges[i], excl, config.page_size);
    let mut a_none = false;
    let mut b_none = false;

    // If the first element is valid, the current range can simply be replaced.
    if let Some(a) = split.0 {
      config.ranges[i] = a;
    } else {
      a_none = true;
    }

    // If the second element is valid, but the first is not, simply replace the
    // current range. Otherwise, insert the new range after the current range.
    // If inserting, increment the index an extra time.
    if let Some(b) = split.1 {
      if a_none {
        config.ranges[i] = b;
      } else if (config.range_count as usize) < MEM_RANGES {
        config
          .ranges
          .copy_within(i..config.range_count as usize, i + 1);
        config.range_count += 1;
        config.ranges[i + 1] = b;
        i += 1;
      } else {
        // TODO: Either we hit a bug or we're not accounting for configurations
        // that create a bunch of memory ranges. Either way, there is probably a
        // more graceful way to handle this.
        debug_assert!(false);
      }
    } else {
      b_none = true;
    }

    // If neither element is valid, remove the current range. Do not increment
    // the index yet.
    if a_none && b_none {
      config
        .ranges
        .copy_within((i + 1)..config.range_count as usize, i);
      config.range_count -= 1;
      continue;
    }

    i += 1;
  }
}

/// @fn split_range(
///       range: &MemoryRange,
///       excl: &MemoryRange,
///       page_size: usize,
///     ) -> (Option<MemoryRange>, Option<MemoryRange>)
/// @brief   Splits a range using an exclusion range.
/// @returns Handles the following cases:
///
///          * If the ranges are mutually exclusive, returns the original range
///            as the first element in the tuple and None for the second.
///
///          * If the exclusion range fully encompasses the range, returns None
///            for both elements of the tuple.
///
///          * If the down page-aligned base, EE, of the exclusion range is
///            greater than the range base, returns a new range in the first
///            element of the tuple with the original base and a new size
///            calculated using EE as the end. Otherwise, returns None in the
///            first element of the tuple.
///
///            If the up page-aligned end, EB, of the exclusion range is less
///            than the range end, returns a new range in the second element of
///            the tuple with EB as the base a new size calcuated using the
///            original end. Otherwise, returns None in the second element of
///            the tuple.
///
///          The last case handles the exclusion range being fully encompassed
///          by the range as well as the exclusion range overlapping either end
///          of the range and handles returning None if the overlap results in
///          empty ranges.
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
      base: base,
      size: range_end - base,
    })
  } else {
    None
  };

  (a, b)
}
