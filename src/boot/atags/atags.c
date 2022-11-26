#include "atags.h"
#include "ros_kernel.h"
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#define ATAG_NONE      0
#define ATAG_CORE      0x54410001
#define ATAG_MEM       0x54410002
#define ATAG_VIDEOTEXT 0x54410003
#define ATAG_RAMDISK   0x54410004
#define ATAG_INITRD2   0x54410005
#define ATAG_SERIAL    0x54410006
#define ATAG_REVISION  0x54410007
#define ATAG_VIDEOLFB  0x54410008
#define ATAG_CMDLINE   0x54410009

#define count_of(a) (sizeof(a) / sizeof((a)[0]))
#define offset_ptr(p, s) ((const void*)(((const uint8_t*)(p)) + (s)))

/**
 * @struct ATAGHeader
 * @brief  Header descriptor for tags.
 */
typedef struct {
  uint32_t size; // Size of tag in 32-bit words
  uint32_t tag;  // Tag identifier
} ATAGHeader;

/**
 * @struct ATAGCore
 * @brief  Core kernel parameters.
 */
typedef struct {
  uint32_t flags;
  uint32_t page_size;
  uint32_t root_dev;
} ATAGCore;

/**
 * @struct ATAGMem
 * @brief  Memory region available to the kernel.
 */
typedef struct {
  uint32_t size; // Size of the region in bytes.
  uint32_t base; // Base address of the region.
} ATAGMem;

/**
 * @struct ATAG
 * @brief  Wrapper struct for a tag.
 */
typedef struct {
  ATAGHeader hdr;
  union {
    ATAGCore core;
    ATAGMem mem;
  } tag;
} ATAG;

static void handle_mem(ROSKernelInit *init, const ATAGMem *mem);

bool read_atags(ROSKernelInit *init, uintptr_t start) {
  const ATAG *p = (const ATAG*)start;

  if (p == NULL) {
    return false;
  }

  // A valid ATAG list must start with CORE.
  if (p->hdr.tag != ATAG_CORE) {
    return false;
  }

  while (p->hdr.tag != ATAG_NONE) {
    switch (p->hdr.tag) {
      case ATAG_MEM:
        handle_mem(init, &p->tag.mem);
        break;
      default:
        break;
    }

    p = offset_ptr(p, p->hdr.size * sizeof(uint32_t));
  }

  return true;
}

/**
 * @fn void handle_mem(ROSKernelInit *init, const ATAGMem *mem)
 * @brief Adds a memory region to the initialization struct.
 * @param[out] init The ROS kernel initialization struct.
 * @param[in] mem   The memory region ATAG.
 */
static void handle_mem(ROSKernelInit *init, const ATAGMem *mem) {
  static const size_t max_regions = count_of(init->mem_regions);
  size_t mem_rgn = 0;

  while (mem_rgn < max_regions && init->mem_regions[mem_rgn].size != 0) {
    ++mem_rgn;
  }

  if (mem_rgn == max_regions) {
    return;
  }

  init->mem_regions[mem_rgn].base = mem->base;
  init->mem_regions[mem_rgn].size = mem->size;
}
