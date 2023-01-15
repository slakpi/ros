use crate::support::{atags, dtb};
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

    if "memory".as_bytes().cmp(dev_type) != cmp::Ordering::Equal {
      return Ok(false);
    }

    Ok(true)
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

      for i in 0..=self.config.range_count as usize {
        if base <= self.config.ranges[i].base {
          self.config.ranges.copy_within(i..self.config.range_count as usize, i + 1);
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
    node_name: &[u8],
    cursor: &mut dtb::DtbCursor,
  ) -> Result<bool, dtb::DtbError> {
    let mut dev_type = (u32::MAX, 0);
    let mut reg = (u32::MAX, 0);

    loop {
      let prop_hdr = match dtb::move_to_next_property(cursor) {
        Some(prop_hdr) => prop_hdr,
        _ => break,
      };

      let prop_name = dtb::get_string_from_table(hdr, prop_hdr.name_offset, cursor)
        .ok_or(dtb::DtbError::InvalidDtb)?;

      if "device_type".as_bytes().cmp(prop_name) == cmp::Ordering::Equal {
        dev_type = (cursor.get_loc(), prop_hdr.prop_size);
      } else if "reg".as_bytes().cmp(prop_name) == cmp::Ordering::Equal {
        reg = (cursor.get_loc(), prop_hdr.prop_size);
      }

      cursor.skip_and_align(prop_hdr.prop_size);
    }

    if dev_type.0 == u32::MAX || reg.0 == u32::MAX {
      return Ok(true);
    }

    if !DtbMemoryScanner::check_device_type(dev_type.0, dev_type.1, cursor)? {
      return Ok(true);
    }

    _ = self.read_reg(reg.0, reg.1, root, cursor)?;

    if self.config.range_count as usize == MEM_RANGES {
      return Ok(false);
    }

    Ok(true)
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
        self.config.ranges.copy_within(i..self.config.range_count as usize, i + 1);
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
/// @brief Initialize the system memory configuration.
/// @param[in] blob        The DTB or ATAGs blob.
/// @param[in] page_size   The memory page size to use.
/// @param[in] kernel_base The location of the kernel in memory.
/// @param[in] kernel_size The size of the kernel image.
pub fn init_memory(blob: usize, page_size: usize, kernel_base: usize, kernel_size: usize) {
  let config = unsafe { &mut MEMORY_CONFIG };

  match dtb::check_dtb(blob) {
    Ok(_) => init_memory_from_dtb(blob, config),
    _ => init_memory_from_atags(blob, config),
  };

  config.page_size = page_size;

  trim_overlapping_ranges(config);

  // Exclude 0 up to the end of the kernel image.
  let kernel = MemoryRange {
    base: 0,
    size: kernel_base + kernel_size,
  };

  exclude_range(config, &kernel);
}

/// @fn init_memory_from_dtb(blob: usize, config: &mut MemoryConfig)
/// @brief Initialize the system memory configuration from a DTB.
/// @param[in] blob The DTB blob.
fn init_memory_from_dtb(blob: usize, config: &mut MemoryConfig) {
  let mut scanner = DtbMemoryScanner { config: config };
  let total_size = match dtb::scan_dtb(blob, &mut scanner) {
    Ok(total_size) => total_size,
    _ => return,
  };

  // Exclude the DTB from configured range.
  let dtb = MemoryRange {
    base: blob,
    size: total_size as usize,
  };

  exclude_range(config, &dtb);
}

/// @fn init_memory_from_atags(blob: usize, config: &mut MemoryConfig)
/// @brief Initialize the system memory configuration from ATAGs.
/// @param[in] blob The ATAGs blob.
fn init_memory_from_atags(blob: usize, config: &mut MemoryConfig) {
  let mut scanner = AtagMemoryScanner { config: config };
  _ = atags::scan_atags(blob, &mut scanner);
}

/// @fn fn trim_overlapping_ranges(config: &mut MemoryConfig)
/// @brief Removes overlapping ranges from the configured ranges.
/// @pre   The configured ranges must be sorted by base address.
/// @param[in] config The current memory configuration.
fn trim_overlapping_ranges(config: &mut MemoryConfig) {
  if config.range_count < 2 {
    return; // Nothing to do.
  }

  let mut i = 0usize;

  loop {
    if i == (config.range_count - 1) as usize {
      return; // No more ranges past this one.
    }

    let a_range = &config.ranges[i];
    let b_range = &config.ranges[i + 1];

    // The ranges are identical. Just remove this range. Do not increment i.
    if a_range.base == b_range.base && a_range.size == b_range.size {
      config.ranges.copy_within((i + 1)..config.range_count as usize, i);
      config.range_count -= 1;
      continue;
    }

    // The end address is NOT part of the range.
    let a_end = a_range.base + a_range.size;
    let b_end = b_range.base + b_range.size;

    if a_range.base <= b_range.base && a_end >= b_end {
    // If this range encompasses the next range remove the next range. Do not
    // increment i.
      if i + 1 < (config.range_count as usize) - 1 {
        config.ranges.copy_within((i + 2)..config.range_count as usize, i + 1);
      }

      config.range_count -= 1;
    } else if a_range.base <= b_range.base && a_end < b_end {
    // If this range overlaps the next range, just reduce this size of this
    // range. If this range's size goes to zero, just remove it. Since the
    // ranges are sorted by base, this is the last case we need to worry about.
    // If we remove this range, do not increment i. Otherwise, move on.
      let new_end = page_align_address_down(b_range.base, config.page_size);

      if new_end <= a_range.base {
        config.ranges.copy_within((i + 1)..config.range_count as usize, i);
        config.range_count -= 1;
      } else {
        config.ranges[i].size = new_end - a_range.base;
        i += 1;
      }
    }
  }
}

/// @fn exclude_range(config: &mut MemoryConfig, excl: &MemoryRange)
/// @brief Excludes a memory range from the configured ranges.
/// @pre   The configured ranges must be sorted by base address.
/// @param[in] config The current memory configuration.
/// @param[in] excl   The exclusion range. Does not need to be page aligned.
fn exclude_range(config: &mut MemoryConfig, excl: &MemoryRange) {
  let excl_end = excl.base + excl.size;
  let count = config.range_count;

  for i in 0..count {
    let range = &mut config.ranges[i as usize];
    let range_end = range.base + range.size;

    // Skip empty ranges.
    if range.size == 0 {
      continue;
    }

    if range.base < excl_end && range_end > excl.base {
    // This range is completely contained in the exclusion range, just zero it
    // out to be trimmed later.
      range.base = 0;
      range.size = 0;
    } else if excl.base < range_end && excl_end > range.base {
    // The exclusion range is completely contained in this range. Need to
    // split this range up.
      if count as usize == MEM_RANGES {
      // This should never happen. If we do not have any room to split the
      // just zero out the whole range and stop (the exclusion can't overlap any
      // other ranges anyway).
        debug_assert!(false);
        range.base = 0;
        range.size = 0;
        break;
      }


    } else if excl.base <= range.base && excl_end > range.base {
    // This range is not completely enclosed, but the exclusion range overlaps
    // the range beginning. Just move the beginning of the range up and reduce
    // the size. If the new base is greater than or equal to the end, zero out
    // the range.
      range.base = page_align_address_up(excl_end, config.page_size);

      if range.base >= range_end {
        range.base = 0;
        range.size = 0;
      } else {
        range.size = range_end - range.base;
      }
    } else if range.base <= excl.base && range_end > excl.base {
    // This range is not completely enclosed, but the exclusion range overlaps
    // the range end. Just move the end of the range down and reduce the size.
    // If the new end is less than or equal to the base, zero out the range.
      let new_end = page_align_address_down(excl.base, config.page_size);

      if new_end <= range.base {
        range.base = 0;
        range.size = 0;
      } else {
        range.size = new_end - range.base;
      }
    }
  }
}

fn trim_empty_ranges(config: &mut MemoryConfig) {

}

fn page_align_address_down(addr: usize, page_size: usize) -> usize {
  addr & (-(page_size as isize) as usize)
}

fn page_align_address_up(addr: usize, page_size: usize) -> usize {
  (addr + (page_size - 1)) & (-(page_size as isize) as usize)
}
