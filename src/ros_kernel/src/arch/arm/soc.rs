//! SoC platform configuration.

use crate::support::dtb;
use core::cmp;

/// Maximum number of SoC memory mappings that can be stored in a configuration.
const SOC_MAPPINGS: usize = 64;

/// Mapping from system peripheral addresses to CPU address.
#[derive(Copy, Clone)]
pub struct SocMapping {
  pub soc_base: usize,
  pub cpu_base: usize,
  pub size: usize,
}

/// Mapping container.
pub struct SocConfig {
  mappings: [SocMapping; SOC_MAPPINGS],
  count: usize,
}

impl SocConfig {
  pub fn new() -> Self {
    SocConfig {
      mappings: [SocMapping {
        soc_base: 0,
        cpu_base: 0,
        size: 0,
      }; SOC_MAPPINGS],
      count: 0,
    }
  }

  /// Get the valid mappings in the container.
  ///
  /// # Returns
  ///
  /// A slice containing the valid mappings.
  pub fn get_mappings(&self) -> &[SocMapping] {
    &self.mappings[0..self.count]
  }

  /// Add a mapping to the container.
  ///
  /// # Parameters
  ///
  /// * `mapping` - The mapping to add to the container.
  pub fn add_mapping(&mut self, mapping: SocMapping) {
    if self.count >= SOC_MAPPINGS {
      return;
    }

    self.mappings[self.count] = mapping;
    self.count += 1;
  }
}

/// Scan the DTB for SoC memory mappings.
///
/// # Parameters
///
/// * `blob` - The DTB address.
///
/// # Returns
///
/// SoC memory mappings found in the DTB, or None if an error occurs. The
/// mapping list may be empty.
pub fn get_soc_memory_layout(blob: usize) -> Option<SocConfig> {
  let mut config = SocConfig::new();
  let reader = dtb::DtbReader::new(blob).ok()?;

  let root_node = reader.get_root_node()?;
  let (cpu_addr_cells, cpu_size_cells) = get_cell_config(&reader, &root_node, 0, 0).ok()?;

  let soc_node = reader.find_child_node(&root_node, "soc")?;
  let (soc_addr_cells, soc_size_cells) =
    get_cell_config(&reader, &soc_node, cpu_addr_cells, cpu_size_cells).ok()?;

  _ = read_soc_mappings(
    &reader,
    &soc_node,
    soc_addr_cells,
    cpu_addr_cells,
    soc_size_cells,
    &mut config,
  )
  .ok()?;

  Some(config)
}

pub fn get_soc_core_count(blob: usize) -> Option<usize> {
  let mut scanner = DtbCpuScanner::new();
  let reader = dtb::DtbReader::new(blob).ok()?;

  _ = reader.scan(&mut scanner).ok()?;

  Some(scanner.count)
}

fn get_cell_config(
  reader: &dtb::DtbReader,
  cursor: &dtb::DtbCursor,
  default_addr_cells: u32,
  default_size_cells: u32,
) -> Result<(u32, u32), dtb::DtbError> {
  let mut tmp_cursor = *cursor;
  let mut addr_cells = default_addr_cells;
  let mut size_cells = default_size_cells;

  while let Some(header) = reader.get_next_property(&mut tmp_cursor) {
    let name = reader
      .get_slice_from_string_table(header.name_offset)
      .ok_or(dtb::DtbError::InvalidDtb)?;

    if "#address-cells".as_bytes().cmp(name) == cmp::Ordering::Equal {
      addr_cells = reader
        .get_u32(&mut tmp_cursor)
        .ok_or(dtb::DtbError::InvalidDtb)?;
    } else if "#size-cells".as_bytes().cmp(name) == cmp::Ordering::Equal {
      size_cells = reader
        .get_u32(&mut tmp_cursor)
        .ok_or(dtb::DtbError::InvalidDtb)?;
    } else {
      reader.skip_and_align(header.size, &mut tmp_cursor);
    }
  }

  Ok((addr_cells, size_cells))
}

fn read_soc_mappings(
  reader: &dtb::DtbReader,
  cursor: &dtb::DtbCursor,
  soc_addr_cells: u32,
  cpu_addr_cells: u32,
  size_cells: u32,
  config: &mut SocConfig,
) -> Result<(), dtb::DtbError> {
  let mut tmp_cursor = *cursor;

  while let Some(header) = reader.get_next_property(&mut tmp_cursor) {
    let name = reader
      .get_slice_from_string_table(header.name_offset)
      .ok_or(dtb::DtbError::InvalidDtb)?;

    if "ranges".as_bytes().cmp(name) == cmp::Ordering::Equal {
      return read_ranges(
        reader,
        &tmp_cursor,
        soc_addr_cells,
        cpu_addr_cells,
        size_cells,
        header.size,
        config,
      );
    } else {
      reader.skip_and_align(header.size, &mut tmp_cursor);
    }
  }

  Ok(())
}

fn read_ranges(
  reader: &dtb::DtbReader,
  cursor: &dtb::DtbCursor,
  soc_addr_cells: u32,
  cpu_addr_cells: u32,
  size_cells: u32,
  prop_size: usize,
  config: &mut SocConfig,
) -> Result<(), dtb::DtbError> {
  let range_size = dtb::DtbReader::get_range_size(soc_addr_cells, cpu_addr_cells, size_cells);

  if (range_size == 0)
    || (prop_size == 0)
    || (prop_size < range_size)
    || (prop_size % range_size != 0)
  {
    return Err(dtb::DtbError::InvalidDtb);
  }

  if soc_addr_cells > cpu_addr_cells {
    return Err(dtb::DtbError::InvalidDtb);
  }

  let mut remaining = prop_size;
  let mut tmp_cursor = *cursor;

  while remaining > 0 {
    let (soc_base, cpu_base, size) = reader
      .get_range(soc_addr_cells, cpu_addr_cells, size_cells, &mut tmp_cursor)
      .ok_or(dtb::DtbError::InvalidDtb)?;

    // Aside from the case where the SoC and CPU have the same address width,
    // we need to also worry about the SoC being 64-bit, the CPU being 32-bit,
    // and vice versa. We've already taken care of the SoC being 64-bit and the
    // CPU being 32-bit above when we verify the SoC cells count is not greater
    // than the CPU cell count. That leaves the SoC being 32-bit and the CPU
    // being 64-bit. In that case, we still just need to verify that the base
    // and size do not overflow 64-bit. This does not guarantee that we will
    // correctly communicate with the SoC, but we will not panic.
    if (cpu_base > usize::MAX as u64) || (soc_base > usize::MAX as u64) {
      continue;
    }

    // The base addresses are known to be less than the maximum platform
    // address. Subtract the bases to get the maximum allowable sizes. If the
    // CPU or SoC size exceeds the maximum, we need to skip this mapping. We
    // should not partially map the SoC.
    let cpu_max_size = (usize::MAX as u64) - cpu_base;
    let soc_max_size = (usize::MAX as u64) - soc_base;
    if size > cpu_max_size || size > soc_max_size {
      continue;
    }

    config.add_mapping(SocMapping {
      soc_base: soc_base as usize,
      cpu_base: cpu_base as usize,
      size: size as usize,
    });

    remaining -= range_size;
  }

  Ok(())
}

/// Scans for DTB CPU nodes.
///
///   NOTE: It would probably be more efficient to add a method to the DTB
///         reader that allows walking the immediate children of a node rather
///         than scanning the whole DTB for CPU nodes. That would allow finding
///         /cpus, then checking for `cpu@x` children.
struct DtbCpuScanner {
  count: usize,
}

impl DtbCpuScanner {
  pub fn new() -> Self {
    DtbCpuScanner {
      count: 0,
    }
  }
}

impl dtb::DtbScanner for DtbCpuScanner {
  /// Check if a node matches `cpu@[0-9]+`.
  ///
  /// # Assumptions
  ///
  /// The digits are not validated. The scanner assumes digits will follow if
  /// the node name starts with `cpu@`.
  fn scan_node(
    &mut self,
    reader: &dtb::DtbReader,
    name: &[u8],
    cursor: &dtb::DtbCursor,
  ) -> Result<bool, dtb::DtbError> {
    // Check if the name is long enough to contain at least `cpu@x`.
    if name.len() < 5 {
      return Ok(true);
    }

    // If long enough, check that the string starts with `cpu@`. Increment the
    // CPU count if it does.
    match "cpu@".as_bytes().cmp(&name[..4]) {
      cmp::Ordering::Equal => self.count += 1,
      _ => {}
    }

    Ok(true)
  }
}
