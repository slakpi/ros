//! SoC platform configuration.

use crate::support::dtb;
use core::cmp;

const SOC_MAPPINGS: usize = 64;

#[derive(Copy, Clone)]
pub struct SocMapping {
  pub soc_base: usize,
  pub cpu_base: usize,
  pub size: usize,
}

pub struct SocConfig {
  mappings: [SocMapping; SOC_MAPPINGS],
  mapping_count: usize,
}

impl SocConfig {
  pub fn new() -> Self {
    SocConfig {
      mappings: [SocMapping {
        soc_base: 0,
        cpu_base: 0,
        size: 0,
      }; SOC_MAPPINGS],
      mapping_count: 0,
    }
  }

  pub fn get_mappings(&self) -> &[SocMapping] {
    &self.mappings[0..self.mapping_count]
  }

  pub fn add_mapping(&mut self, mapping: SocMapping) {
    if self.mapping_count >= SOC_MAPPINGS {
      return;
    }

    self.mappings[self.mapping_count] = mapping;
    self.mapping_count += 1;
  }
}

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
  property_size: usize,
  config: &mut SocConfig,
) -> Result<(), dtb::DtbError> {
  let range_size = dtb::DtbReader::get_range_size(soc_addr_cells, cpu_addr_cells, size_cells);

  if property_size % range_size != 0 {
    return Err(dtb::DtbError::InvalidDtb);
  }

  let mut remaining = property_size;
  let mut tmp_cursor = *cursor;

  while remaining > 0 {
    let (soc_base, cpu_base, size) = reader
      .get_range(soc_addr_cells, cpu_addr_cells, size_cells, &mut tmp_cursor)
      .ok_or(dtb::DtbError::InvalidDtb)?;

    config.add_mapping(SocMapping {
      soc_base,
      cpu_base,
      size,
    });

    remaining -= range_size;
  }

  Ok(())
}
