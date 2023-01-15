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
  /// @brief   Check if this node describes a memory device.
  /// @param[in] loc    The location of the property data in the node.
  /// @param[in] size   The size of the property.
  /// @param[in] cursor The DTB cursor.
  /// @returns Ok(true) if this is a memory device, Ok(false) if it is not, or
  ///          Err if an error is encountered.
  fn check_device_type(
    loc: u32,
    size: u32,
    cursor: &mut dtb::DtbCursor,
  ) -> Result<bool, dtb::DtbError> {
    if size == 0 {
      return Err(dtb::DtbError::InvalidDtb);
    }

    let old_loc = cursor.get_loc();
    cursor.set_loc(loc);

    let dev_type = cursor
      .get_u8_slice(size - 1)
      .ok_or(dtb::DtbError::InvalidDtb)?;

    if "memory".as_bytes().cmp(dev_type) != cmp::Ordering::Equal {
      return Ok(false);
    }

    cursor.set_loc(old_loc);

    Ok(true)
  }

  /// @fn read_reg(
  ///       &mut self,
  ///       loc: u32,
  ///       size: u32,
  ///       root: &dtb::DtbRoot,
  ///       cursor: &mut dtb::DtbCursor,
  ///     ) -> Result<(), dtb::DtbError>
  /// @brief Read a reg property.
  /// @param[in] loc    The location of the property data in the node.
  /// @param[in] size   The size of the property data.
  /// @param[in] root   The root node describing reg property layout.
  /// @param[in] cursor The DTB cursor.
  /// @returns Ok if the reg property is valid or Err if an error is
  ///          encountered.
  fn read_reg(
    &mut self,
    loc: u32,
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
    let old_loc = cursor.get_loc();
    cursor.set_loc(loc);

    for _ in 0..reg_count {
      let range = &mut self.config.ranges[self.config.range_count as usize];
      (range.base, range.size) = cursor
        .get_reg(root.addr_cells, root.size_cells)
        .ok_or(dtb::DtbError::InvalidDtb)?;
      self.config.range_count += 1;
    }

    cursor.set_loc(old_loc);

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
    let range = &mut self.config.ranges[self.config.range_count as usize];
    range.base = mem.base as usize;
    range.size = mem.size as usize;
    self.config.range_count += 1;

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
}

/// @fn init_memory_from_dtb(blob: usize, config: &mut MemoryConfig)
/// @brief Initialize the system memory configuration from a DTB.
/// @param[in] blob The DTB blob.
fn init_memory_from_dtb(blob: usize, config: &mut MemoryConfig) {
  let mut scanner = DtbMemoryScanner { config: config };

  _ = dtb::scan_dtb(blob, &mut scanner);
}

/// @fn init_memory_from_atags(blob: usize, config: &mut MemoryConfig)
/// @brief Initialize the system memory configuration from ATAGs.
/// @param[in] blob The ATAGs blob.
fn init_memory_from_atags(blob: usize, config: &mut MemoryConfig) {
  let mut scanner = AtagMemoryScanner { config: config };

  _ = atags::scan_atags(blob, &mut scanner);
}
