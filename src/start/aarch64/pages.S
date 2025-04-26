//! AArch64 Page Table Setup

#include "abi.h"
#include "mmu.h"

// Page descriptor flags. See D8.3.2. Note: Bits 58:55 are reserved for
// software use. Bit 6 is zero to deny access to EL0. Memory is RW if bit 7 is
// 0, RO otherwise.
#define MM_TYPE_PAGE_TABLE 0x3
#define MM_TYPE_PAGE       0x3
#define MM_TYPE_BLOCK      0x1
#define MM_ACCESS_FLAG     (1 << 10)
#define MM_ACCESS_RW       (0b00 << 6)
#define MM_ACCESS_RO       (0b10 << 6)

#define MMU_NORMAL_RO_FLAGS (MM_TYPE_BLOCK | (MT_NORMAL_NC << 2) | MM_ACCESS_RO | MM_ACCESS_FLAG)
#define MMU_NORMAL_RW_FLAGS (MM_TYPE_BLOCK | (MT_NORMAL_NC << 2) | MM_ACCESS_RW | MM_ACCESS_FLAG)
#define MMU_DEVICE_RO_FLAGS                                                                        \
  (MM_TYPE_BLOCK | (MT_DEVICE_nGnRnE << 2) | MM_ACCESS_RO | MM_ACCESS_FLAG)
#define MMU_DEVICE_RW_FLAGS                                                                        \
  (MM_TYPE_BLOCK | (MT_DEVICE_nGnRnE << 2) | MM_ACCESS_RW | MM_ACCESS_FLAG)

/// 2 MiB section virtual address layout:
///
///   +---------------+--------+--------+--------+--------------------+
///   | / / / / / / / |   L1   |   L2   |   L3   |       Offset       |
///   +---------------+--------+--------+--------+--------------------+
///   63             48       39       30       21                    0
///
/// 4 KiB page virtual address layout:
///
///   +---------------+--------+--------+--------+--------+-----------+
///   | / / / / / / / |   L1   |   L2   |   L3   |   L4   |  Offset   |
///   +---------------+--------+--------+--------+--------+-----------+
///   63             48       39       30       21       12           0
#define PAGE_SHIFT      12
#define TABLE_SHIFT     9
#define SECTION_SHIFT   (PAGE_SHIFT + TABLE_SHIFT)
#define SECTION_SIZE    (1 << SECTION_SHIFT)	
#define TABLE_ENTRY_CNT (1 << TABLE_SHIFT)

#define L1_SHIFT (PAGE_SHIFT + (3 * TABLE_SHIFT))
#define L2_SHIFT (PAGE_SHIFT + (2 * TABLE_SHIFT))


/*----------------------------------------------------------------------------*/
/// Create the initial kernel page tables.
///
/// # Parameters
///
/// * x0 - The base of the blob.
/// * x1 - The size of the DTB or 0 if the blob is not a DTB.
///
/// # Description
///
/// Maps the kernel and, as necessary, the DTB into 2 MiB sections. The kernel
/// will re-map the pages after determining the memory layout.
.global mmu_create_kernel_page_tables
mmu_create_kernel_page_tables:
  fn_entry
  stp     x19, x20, [sp, #-16]!
  stp     x21, x22, [sp, #-16]!
  stp     x23, x24, [sp, #-16]!

  mov     x19, x0

// Align the blob size on a section.
  mov     x0, x1
  bl      section_align_size
  mov     x20, x0

// Align the size of the kernel image on a section.
  adrp    x0, __kernel_end
  bl      section_align_size
  mov     x21, x0

// Clear the page tables.
  adrp    x0, __kernel_pages_start
  mov     x1, #0
  ldr     x2, =__kernel_pages_size
  bl      memset

  adrp    x0, __kernel_id_pages_start
  mov     x1, #0
  ldr     x2, =__kernel_id_pages_size
  bl      memset

// Create the L1 and L2 page tables.
  adrp    x0, __kernel_pages_start
  ldr     x1, =__virtual_start
  bl      init_tables
  mov     x22, x0

  adrp    x0, __kernel_id_pages_start
  mov     x1, #0
  bl      init_tables
  mov     x23, x0

// Map the kernel area as RW normal memory.
//
//   TODO: The code should probably be separate from the stack and page tables
//         to prevent the code from being re-written.
  mov     x0, x22
  mov     x1, #0
  ldr     x2, =__virtual_start
  add     x3, x2, x21
  sub     x3, x3, #1
  mov     x4, #MMU_NORMAL_RW_FLAGS
  bl      map_block

  mov     x0, x23
  mov     x1, #0
  mov     x2, #0
  add     x3, x2, x21
  sub     x3, x3, #1
  mov     x4, #MMU_NORMAL_RW_FLAGS
  bl      map_block

// Map the DTB area as RO normal memory. Skip this if the DTB size is zero.
// Do not need to create an identity map. The kernel will switch to virtual
// addresses before the DTB is needed.
  cbz     x20, skip_dtb_mapping

  mov     x0, x22
  mov     x1, #0
  add     x1, x1, x19
  ldr     x2, =__virtual_start
  add     x2, x2, x19
  add     x3, x2, x20
  sub     x3, x3, #1
  mov     x4, #MMU_NORMAL_RO_FLAGS
  bl      map_block

skip_dtb_mapping:
  fn_exit
  ret


/*----------------------------------------------------------------------------*/
/// Section-align the size with the next section higher.
///
/// # Parameters
///
/// * x0 - The size to align.
///
/// # Returns
///
/// The section-aligned size.
section_align_size:
// no fn_entry required.

  mov     x9, #SECTION_SIZE - 1
  add     x0, x0, x9

  mov     x9, #SECTION_SIZE
  neg     x9, x9
  and     x0, x0, x9

// no fn_exit required.
  ret


/*----------------------------------------------------------------------------*/
/// Initialize L1 and L2 page tables for the first 1 GiB of the physical address
/// space.
///
/// # Parameters
///
/// * x0 - The base address of the L1 table.
/// * x1 - The base address of the virtual address space.
///
/// # Returns
///
/// Returns the base address of the L3 page table.
init_tables:
  fn_entry

  mov     x2, #L1_SHIFT
  bl      create_table_entry

  mov     x2, #L2_SHIFT
  bl      create_table_entry

  fn_exit
  ret


/*----------------------------------------------------------------------------*/
/// Helper for `init_tables`. Do not call directly.
///
/// # Parameters
///
/// * x0 - The base address of the L1 or L2 table.
/// * x1 - The base address of the virtual address space.
/// * x2 - The shift specifying the L1 or L2 table.
///
/// # Returns
///
/// The address of the next page after the table.
create_table_entry:
// no fn_entry required.

// Shift the virtual address down and mask it with the entry count to get the
// entry index.
  lsr     x9, x1, x2
  and     x9, x9, #TABLE_ENTRY_CNT - 1

// Get the pointer to the table at the next page. Assume the address is page-
// aligned, so the offset bits are already zero.
  ldr     x10, =__page_size
  add     x10, x0, x10

// Create the entry.
  orr     x10, x10, #MM_TYPE_PAGE_TABLE	
  str     x10, [x0, x9, lsl #3]

// Return the address of the next page table.
  ldr     x10, =__page_size
  add     x0, x0, x10

// no fn_exit required.
  ret


/*----------------------------------------------------------------------------*/
/// Map a block of 2 MiB sections to a L3 translation table.
///
/// # Parameters
///
/// * x0 - The base address of the L3 table.
/// * x1 - The base physical address.
/// * x2 - The base virtual address.
/// * x3 - The last virtual address.
/// * x4 - The entry flags.
map_block:
// no fn_entry required.

  lsr     x2, x2, #SECTION_SHIFT
  and     x2, x2, #TABLE_ENTRY_CNT - 1
  lsr     x3, x3, #SECTION_SHIFT
  and     x3, x3, #TABLE_ENTRY_CNT - 1
  lsr     x1, x1, #SECTION_SHIFT
  orr     x1, x4, x1, lsl #SECTION_SHIFT
1:
  str     x1, [x0, x2, lsl #3]
  add     x2, x2, #1
  add     x1, x1, #SECTION_SIZE
  cmp     x2, x3
  b.ls    1b

// no fn_exit required.
  ret
