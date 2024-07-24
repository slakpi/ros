//! ARM CPU Peripheral Utilities

use crate::support::{dtb, hash, hash_map};
use core::cmp;
use core::convert::TryFrom;

/// Maximum number of cores supported for an ARM SoC.
pub const MAX_CORES: usize = 512;

/// Length of a core type name.
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

/// Core information.
#[derive(Copy, Clone)]
pub struct Core {
  id: usize,
  core_type: [u8; CPU_TYPE_LEN],
  enable_method: CoreEnableMethod,
  release_addr: usize,
}

impl Core {
  pub fn get_id(&self) -> usize {
    self.id
  }

  pub fn get_core_type(&self) -> &[u8] {
    &self.core_type
  }

  pub fn get_enable_method(&self) -> CoreEnableMethod {
    self.enable_method
  }

  pub fn get_release_addr(&self) -> usize {
    self.release_addr
  }
}

/// System CPU configuration.
pub struct CpuConfig {
  cores: [Core; MAX_CORES],
  count: usize,
}

impl CpuConfig {
  pub const fn new() -> Self {
    CpuConfig {
      cores: [Core {
        id: 0,
        core_type: [0; CPU_TYPE_LEN],
        enable_method: CoreEnableMethod::Invalid,
        release_addr: 0,
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

  pub fn get_cores(&self) -> &[Core] {
    &self.cores[..self.count]
  }
}

/// Tags for expected properties and values.
enum StringTag {
  DtbPropCompatible,
  DtbPropEnableMethod,
  DtbPropCpuReleaseAddr,
  DtbPropReg,
  DtbValueSpinTable,
  DtbValuePsci,
}

type StringMap<'map> = hash_map::HashMap<&'map [u8], StringTag, hash::BuildFnv1aHasher, 31>;

/// Scans for DTB CPU nodes.
struct DtbCpuScanner<'config> {
  config: &'config mut CpuConfig,
  string_map: StringMap<'config>,
}

impl<'config> DtbCpuScanner<'config> {
  /// Construct a new DtbCpuScanner.
  pub fn new(config: &'config mut CpuConfig) -> Self {
    DtbCpuScanner {
      config,
      string_map: Self::build_string_map(),
    }
  }

  /// Build a string map for the scanner.
  ///
  /// # Returns
  ///
  /// A new string map for the expected properties and values.
  fn build_string_map() -> StringMap<'config> {
    let mut string_map = StringMap::with_hasher_factory(hash::BuildFnv1aHasher {});
    string_map.insert("compatible".as_bytes(), StringTag::DtbPropCompatible);
    string_map.insert("enable-method".as_bytes(), StringTag::DtbPropEnableMethod);
    string_map.insert(
      "cpu-release-addr".as_bytes(),
      StringTag::DtbPropCpuReleaseAddr,
    );
    string_map.insert("reg".as_bytes(), StringTag::DtbPropReg);
    string_map.insert("spin-table".as_bytes(), StringTag::DtbValueSpinTable);
    string_map.insert("psci".as_bytes(), StringTag::DtbValuePsci);
    string_map
  }

  /// Scan a `cpu@N` node and add it to the set of known cores.
  ///
  /// # Parameters
  ///
  /// * `reader` - The DTB reader.
  /// * `cursor` - The current position in the DTB.
  ///
  /// # Returns
  ///
  /// Returns Ok if able to read the node, otherwise a DTB error.
  fn scan_cpu_node(
    &mut self,
    reader: &dtb::DtbReader,
    cursor: &dtb::DtbCursor,
  ) -> Result<(), dtb::DtbError> {
    let mut tmp_cursor = *cursor;
    let mut core = Core {
      id: usize::MAX,
      core_type: [0; CPU_TYPE_LEN],
      enable_method: CoreEnableMethod::Invalid,
      release_addr: 0,
    };

    while let Some(header) = reader.get_next_property(&mut tmp_cursor) {
      let tag = self.string_map.find(&header.name);

      match tag {
        Some(StringTag::DtbPropCompatible) => {
          Self::read_compatible(&mut core.core_type, reader, &mut tmp_cursor)?;
        }
        Some(StringTag::DtbPropEnableMethod) => {
          core.enable_method = Self::read_enable_method(reader, &mut tmp_cursor, &self.string_map)?;
        }
        Some(StringTag::DtbPropCpuReleaseAddr) => {
          core.release_addr = Self::read_cpu_release_addr(header.size, reader, &mut tmp_cursor)?;
        }
        Some(StringTag::DtbPropReg) => {
          core.id = Self::read_reg(reader, &mut tmp_cursor)?;
        }
        _ => reader.skip_and_align(header.size, &mut tmp_cursor),
      }
    }

    if core.id >= MAX_CORES {
      return Err(dtb::DtbError::InvalidDtb);
    }

    self.config.cores[core.id] = core;
    self.config.count += 1;

    Ok(())
  }

  /// Read the `compatible` property with the core name.
  ///
  /// # Parameters
  ///
  /// * `core_type` - The slice to receive the string.
  /// * `reader` - The DTB reader.
  /// * `cursor` - The current position in the DTB.
  ///
  /// # Returns
  ///
  /// Returns Ok if able to read the property, otherwise a DTB error.
  fn read_compatible(
    core_type: &mut [u8],
    reader: &dtb::DtbReader,
    cursor: &mut dtb::DtbCursor,
  ) -> Result<(), dtb::DtbError> {
    let compatible = reader
      .get_null_terminated_u8_slice(cursor)
      .ok_or(dtb::DtbError::InvalidDtb)?;
    reader.skip_and_align(1, cursor);

    let len = cmp::min(compatible.len(), core_type.len());
    core_type[..len].clone_from_slice(&compatible[..len]);

    Ok(())
  }

  /// Read the `enable-method` property.
  ///
  /// # Parameters
  ///
  /// * `reader` - The DTB reader.
  /// * `cursor` - The current position in the DTB.
  ///
  /// # Returns
  ///
  /// Returns Ok with the enable method if valid, otherwise a DTB error.
  fn read_enable_method(
    reader: &dtb::DtbReader,
    cursor: &mut dtb::DtbCursor,
    string_map: &StringMap,
  ) -> Result<CoreEnableMethod, dtb::DtbError> {
    let enable_method = reader
      .get_null_terminated_u8_slice(cursor)
      .ok_or(dtb::DtbError::InvalidDtb)?;
    reader.skip_and_align(1, cursor);

    let tag = string_map
      .find(&enable_method)
      .ok_or(dtb::DtbError::UnknownValue)?;

    match tag {
      StringTag::DtbValueSpinTable => Ok(CoreEnableMethod::SpinTable),
      _ => Err(dtb::DtbError::UnsupportedValue),
    }
  }

  /// Read the `cpu-release-addr` property.
  ///
  /// # Parameters
  ///
  /// * `size` - The size of the property's value.
  /// * `reader` - The DTB reader.
  /// * `cursor` - The current position in the DTB.
  ///
  /// # Description
  ///
  ///     NOTE: The `cpu-release-addr` property SHOULD always be 64-bit, however
  ///           there exist DTBs that use 32-bit addresses.
  ///           https://devicetree-specification.readthedocs.io/en/stable/devicenodes.html#cpus-cpu-node-properties
  ///
  /// # Returns
  ///
  /// Returns Ok with the core release address if valid, otherwise a DTB error.
  fn read_cpu_release_addr(
    size: usize,
    reader: &dtb::DtbReader,
    cursor: &mut dtb::DtbCursor,
  ) -> Result<usize, dtb::DtbError> {
    match size {
      4 => Ok(reader.get_u32(cursor).ok_or(dtb::DtbError::InvalidDtb)? as usize),
      8 => {
        let addr = reader.get_u64(cursor).ok_or(dtb::DtbError::InvalidDtb)?;
        usize::try_from(addr).ok().ok_or(dtb::DtbError::InvalidDtb)
      }
      _ => Err(dtb::DtbError::UnsupportedValue),
    }
  }

  /// Read the `reg` property with the core number.
  ///
  /// # Parameters
  ///
  /// * `reader` - The DTB reader.
  /// * `cursor` - The current position in the DTB.
  ///
  /// # Returns
  ///
  /// Returns Ok with the core number if valid, otherwise a DTB error.
  fn read_reg(
    reader: &dtb::DtbReader,
    cursor: &mut dtb::DtbCursor,
  ) -> Result<usize, dtb::DtbError> {
    let core_id = reader.get_u32(cursor).ok_or(dtb::DtbError::InvalidDtb)? as usize;
    Ok(core_id)
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
  for core in config.get_cores() {
    match core.enable_method {
      CoreEnableMethod::SpinTable => {}
      _ => return false,
    }
  }

  true
}
