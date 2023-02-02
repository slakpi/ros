/// @file atags.rs
/// @brief   ATAGs Utilities
/// @details http://www.simtec.co.uk/products/SWLINUX/files/booting_article.html#appendix_tag_reference

const ATAG_NONE: u32 = 0;
const ATAG_CORE: u32 = 0x54410001;
const ATAG_MEM: u32 = 0x54410002;
const _ATAG_VIDEOTEXT: u32 = 0x54410003;
const _ATAG_RAMDISK: u32 = 0x54410004;
const _ATAG_INITRD2: u32 = 0x54410005;
const _ATAG_SERIAL: u32 = 0x54410006;
const _ATAG_REVISION: u32 = 0x54410007;
const _ATAG_VIDEOLFB: u32 = 0x54410008;
const _ATAG_CMDLINE: u32 = 0x54410009;

pub enum AtagError {
  InvalidAtagList,
}

/// @struct AtagHeader
/// @brief  Header for all ATAG entries.
/// @var size Number of 32-bit integers in the ATAG.
/// @var tag  The tag identifier.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct AtagHeader {
  pub size: u32,
  pub tag: u32,
}

/// @struct AtagCore
/// @brief  The CORE ATAG.
/// @var flags     ??? flags that indicate, you know, things ???
/// @var page_size ??? virtual memory page size ???
/// @var root_dev  ??? the root device something something ???
#[repr(C)]
#[derive(Copy, Clone)]
pub struct AtagCore {
  pub flags: u32,
  pub page_size: u32,
  pub root_dev: u32,
}

/// @struct AtagMem
/// @brief  The MEM ATAG.
/// @var size Size of the memory region in bytes.
/// @var base Base address of the region.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct AtagMem {
  pub size: u32,
  pub base: u32,
}

/// @struct AtagData
/// @brief  Overlay of tag data.
#[repr(C)]
#[derive(Copy, Clone)]
pub union AtagData {
  pub core: AtagCore,
  pub mem: AtagMem,
}

/// @struct Atag
/// @brief  Full ATAG with header and overlaid data.
/// @var hdr The ATAG header.
/// @var tag Overlaid ATAG data.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct Atag {
  pub hdr: AtagHeader,
  pub data: AtagData,
}

/// @trait AtagScanner
/// @brief ATAG scanner trait.
pub trait AtagScanner {
  /// @fn scan_core_tag
  /// @brief   Scans a CORE tag.
  /// @param[in] core The CORE tag data.
  /// @returns Ok(true) if scanning should continue, Ok(false) if scanning
  ///          stop, or Err if an error is encountered.
  fn scan_core_tag(&mut self, _core: &AtagCore) -> Result<bool, AtagError> {
    Ok(true) // Just skip by default.
  }

  /// @fn scan_mem_tag
  /// @brief   Scans a MEM tag.
  /// @param[in] mem The MEM tag data.
  /// @returns Ok(true) if scanning should continue, Ok(false) if scanning
  ///          stop, or Err if an error is encountered.
  fn scan_mem_tag(&mut self, _mem: &AtagMem) -> Result<bool, AtagError> {
    Ok(true) // Just skip by default.
  }
}

pub fn scan_atags(blob: usize, scanner: &mut impl AtagScanner) -> Result<(), AtagError> {
  let mut ptr = blob as *const Atag;
  let mut atag = unsafe { *ptr };

  // The ATAG list must start with a CORE tag.
  if atag.hdr.tag != ATAG_CORE {
    return Err(AtagError::InvalidAtagList);
  }

  loop {
    // Stop on a NONE tag or if the scanner wants to stop. Otherwise, keep
    // going.
    let keep_going = unsafe {
      match atag.hdr.tag {
        ATAG_NONE => false,
        ATAG_CORE => scanner.scan_core_tag(&atag.data.core)?,
        ATAG_MEM => scanner.scan_mem_tag(&atag.data.mem)?,
        _ => true,
      }
    };

    if !keep_going {
      break;
    }

    // The size in an ATAG is the number of 32-bit words in the tag, not bytes.
    ptr = unsafe { (ptr as *const u32).add(atag.hdr.size as usize) } as *const Atag;
    atag = unsafe { *ptr };
  }

  Ok(())
}
