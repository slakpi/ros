use crate::support::dtb;
use core::cmp;

const MEM_RANGES: usize = 64;

#[derive(Copy, Clone)]
struct MemoryRange {
  base: usize,
  size: usize,
}

struct MemoryConfig {
  ranges: [MemoryRange; MEM_RANGES],
  range_count: u8,
}

struct MemoryScanner<'mem> {
  config: &'mem mut MemoryConfig,
}

impl<'mem> MemoryScanner<'mem> {
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

  fn read_reg(
    &mut self,
    loc: u32,
    size: u32,
    root: &dtb::DtbRoot,
    cursor: &mut dtb::DtbCursor,
  ) -> Result<(), dtb::DtbError> {
    let reg_size = (root.addr_cells * 4) + (root.size_cells * 4);

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

impl<'mem> dtb::DtbScanner for MemoryScanner<'mem> {
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

      let prop_name =
        dtb::get_string_from_table(hdr, prop_hdr.name_offset, cursor)
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

    if !MemoryScanner::check_device_type(dev_type.0, dev_type.1, cursor)? {
      return Ok(true);
    }

    _ = self.read_reg(reg.0, reg.1, root, cursor)?;

    if self.config.range_count as usize == MEM_RANGES {
      return Ok(false);
    }

    Ok(true)
  }
}

pub fn init_memory(blob: usize) {
  let mut config = MemoryConfig {
    ranges: [MemoryRange { base: 0, size: 0 }; MEM_RANGES],
    range_count: 0,
  };

  let mut scanner = MemoryScanner {
    config: &mut config,
  };

  _ = dtb::scan_dtb(blob, &mut scanner);
}
