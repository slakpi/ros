use super::kernel_init::ROSKernelInit;
use core::mem::ManuallyDrop;

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

/// @struct ATAGHeader
/// @brief Header for all ATAG entries.
/// @var size Number of 32-bit integers in the ATAG.
/// @var tag  The tag identifier.
#[repr(C)]
struct ATAGHeader {
  size: u32,
  tag: u32,
}

/// @struct ATAGCore
/// @brief The CORE ATAG.
/// @var flags     ??? flags that indicate, you know, things ???
/// @var page_size ??? virtual memory page size ???
/// @var root_dev  ??? the root device something something ???
#[repr(C)]
struct ATAGCore {
  flags: u32,
  page_size: u32,
  root_dev: u32,
}

/// @struct ATAGMem
/// @brief The MEM ATAG.
/// @var size Size of the memory region in bytes.
/// @var base Base address of the region.
#[repr(C)]
struct ATAGMem {
  size: u32,
  base: u32,
}

/// @struct ATAGData
/// @brief Overlay of tag data.
#[repr(C)]
union ATAGData {
  core: ManuallyDrop<ATAGCore>,
  mem: ManuallyDrop<ATAGMem>,
}

/// @struct ATAG
/// @brief Full ATAG with header and overlaid data.
/// @var hdr The ATAG header.
/// @var tag Overlaid ATAG data.
#[repr(C)]
struct ATAG {
  hdr: ATAGHeader,
  tag: ATAGData,
}

/// @fn read_atags(init: &mut ROSKernelInit, blob: usize) -> bool
/// @brief   Read ATAGs provided by the bootloader.
/// @param[in] init The kernel initialization struct to fill out.
/// @param[in] blob Pointer to the ATAGs blob.
/// @returns True if able to read ATAGs, false if the blob is not an ATAG list.
pub fn read_atags(init: &mut ROSKernelInit, blob: usize) -> bool {
  unsafe {
    let mut ptr = blob as *const u32;
    let mut hdr = ptr as *const ATAGHeader;

    // The ATAG list must start with CORE.
    if (*hdr).tag != ATAG_CORE {
      return false;
    }

    loop {
      let atag = hdr as *const ATAG;

      match (*atag).hdr.tag {
        ATAG_NONE => break,
        ATAG_MEM => read_mem_atag(init, &(*atag).tag.mem),
        _ => {}
      }

      // Offset ptr. The size field is the number of 32-bit integers in the tag.
      ptr = ptr.offset((*atag).hdr.size as isize);
      hdr = ptr as *const ATAGHeader;
    }
  }

  true
}

/// @fn read_mem_atag(init: &mut ROSKernelInit, tag: &ATAGMem)
/// @brief Add a memory region to the kernel initialization struct.
/// @param[in] init The kernel initialization struct.
/// @param[in] tag  The memory region ATAG.
fn read_mem_atag(init: &mut ROSKernelInit, tag: &ATAGMem) {
  for mut rgn in init.memory_regions {
    if rgn.size == 0 {
      rgn.size = tag.size as usize;
      rgn.base = tag.base as usize;
      break;
    }
  }
}
