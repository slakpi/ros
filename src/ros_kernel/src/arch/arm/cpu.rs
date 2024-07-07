//! ARM CPU Peripheral Utilities

use crate::support::dtb;
use core::cmp;

/// Maximum number of cores supported for an ARM SoC.
pub const MAX_CORES: usize = 512;

pub const CPU_TYPE_LEN: usize = 64;

/// Method used to enable a core.
///
/// Spin tables park each core in a loop watching a specific memory address.
/// A core is enabled by writing the desired address to begin executing to the
/// watch address.
///
/// PSCI is the Power State Coordination Interface.
#[derive(Copy, Clone)]
pub enum CoreEnableMethod {
  Invalid,
  SpinTable,
  Psci,
}

#[derive(Copy, Clone)]
pub struct Core {
  id: usize,
  cpu_type: [u8; CPU_TYPE_LEN],
  enable_method: CoreEnableMethod,
  cpu_release_addr: usize,
}

impl Core {
  pub fn get_cpu_type(&self) -> &[u8] {
    &self.cpu_type
  }

  pub fn get_enable_method(&self) -> CoreEnableMethod {
    self.enable_method
  }

  pub fn get_cpu_release_addr(&self) -> usize {
    self.cpu_release_addr
  }
}

#[derive(Copy, Clone)]
pub struct CpuConfig {
  cores: [Core; MAX_CORES],
  count: usize,
}

impl CpuConfig {
  pub const fn new() -> Self {
    CpuConfig {
      cores: [Core {
        id: 0,
        cpu_type: [0; CPU_TYPE_LEN],
        enable_method: CoreEnableMethod::Invalid,
        cpu_release_addr: 0,
      }; MAX_CORES],
      count: 0,
    }
  }

  pub fn is_empty(&self) -> bool {
    self.count == 0
  }

  pub fn len(&self) -> usize {
    self.count
  }

  pub fn cores(&self) -> &[Core] {
    &self.cores[..self.count]
  }
}

/// Scans for DTB CPU nodes.
struct DtbCpuScanner<'config> {
  config: &'config mut CpuConfig,
}

impl<'config> DtbCpuScanner<'config> {
  pub fn new(config: &'config mut CpuConfig) -> Self {
    DtbCpuScanner {
      config,
    }
  }

  fn scan_cpu_node(
    &mut self,
    reader: &dtb::DtbReader,
    cursor: &dtb::DtbCursor,
  ) -> Result<(), dtb::DtbError> {
    let mut tmp_cursor = *cursor;
    let mut core = Core {
      id: usize::MAX,
      cpu_type: [0; CPU_TYPE_LEN],
      enable_method: CoreEnableMethod::Invalid,
      cpu_release_addr: 0,
    };

    while let Some(header) = reader.get_next_property(&mut tmp_cursor) {
      let name = reader
        .get_slice_from_string_table(header.name_offset)
        .ok_or(dtb::DtbError::InvalidDtb)?;

      if "compatible".as_bytes().cmp(name) == cmp::Ordering::Equal {
        let cpu_type = reader
          .get_null_terminated_u8_slice(&mut tmp_cursor)
          .ok_or(dtb::DtbError::InvalidDtb)?;
        reader.skip_and_align(1, &mut tmp_cursor);
        let len = cmp::min(CPU_TYPE_LEN - 1, cpu_type.len());
        core.cpu_type[..len].clone_from_slice(&cpu_type[..len]);
      } else if "enable-method".as_bytes().cmp(name) == cmp::Ordering::Equal {
        let enable_method = reader
          .get_null_terminated_u8_slice(&mut tmp_cursor)
          .ok_or(dtb::DtbError::InvalidDtb)?;
        reader.skip_and_align(1, &mut tmp_cursor);
        if "spin-table".as_bytes().cmp(enable_method) == cmp::Ordering::Equal {
          core.enable_method = CoreEnableMethod::SpinTable;
        } else if "psci".as_bytes().cmp(enable_method) == cmp::Ordering::Equal {
          core.enable_method = CoreEnableMethod::Psci;
        }
      } else if "cpu-release-addr".as_bytes().cmp(name) == cmp::Ordering::Equal {
        // Note: The `cpu-release-addr` property is always 64-bit.
        //       https://devicetree-specification.readthedocs.io/en/stable/devicenodes.html#cpus-cpu-node-properties
        let cpu_release_addr = reader
          .get_u64(&mut tmp_cursor)
          .ok_or(dtb::DtbError::InvalidDtb)?;

        if cpu_release_addr > usize::MAX as u64 {
          return Err(dtb::DtbError::InvalidDtb);
        }

        core.cpu_release_addr = cpu_release_addr as usize;
      } else if "reg".as_bytes().cmp(name) == cmp::Ordering::Equal {
        core.id = reader
          .get_u32(&mut tmp_cursor)
          .ok_or(dtb::DtbError::InvalidDtb)? as usize;
      } else {
        reader.skip_and_align(header.size, &mut tmp_cursor);
      }
    }

    if core.id >= MAX_CORES {
      return Err(dtb::DtbError::InvalidDtb);
    }

    self.config.cores[core.id] = core;
    self.config.count += 1;

    Ok(())
  }
}

impl<'config> dtb::DtbScanner for DtbCpuScanner<'config> {
  fn scan_node(
    &mut self,
    reader: &dtb::DtbReader,
    name: &[u8],
    cursor: &dtb::DtbCursor,
  ) -> Result<bool, dtb::DtbError> {
    if name.len() >= 5 && "cpu@".as_bytes().cmp(&name[..4]) == cmp::Ordering::Equal {
      _ = self.scan_cpu_node(reader, cursor)?;
    }

    Ok(true)
  }
}

/// Get the CPU configuration.
///
/// # Parameters
///
/// * `config` - The CPU configuration.
/// * `blob` - The DTB address.
///
/// # Assumptions
///
/// Assumes the configuration is empty.
///
/// # Returns
///
/// True if able to read the CPU configuration and at least one valid CPU is
/// provided by the system, false otherwise.
pub fn get_cpu_config(config: &mut CpuConfig, blob: usize) -> bool {
  debug_assert!(config.is_empty());

  let mut scanner = DtbCpuScanner::new(config);

  let reader = match dtb::DtbReader::new(blob) {
    Ok(r) => r,
    _ => return false,
  };

  if !reader.scan(&mut scanner).is_ok() {
    return false;
  }

  if config.is_empty() {
    return false;
  }

  // Validate that the enable method for each core is supported.
  for core in config.cores() {
    match core.enable_method {
      CoreEnableMethod::SpinTable => {}
      _ => return false,
    }
  }

  true
}
