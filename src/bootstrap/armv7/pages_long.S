//! ARMv7a Page Table Bootstrap with Large Physical Address Extensions
//!
//!   NOTE: Currently this code only configures a 3:1 split. It should be
//!         generalized to support 2:2 and 1:3 splits.

#include "abi.h"

/// TTBCR value. See B4.1.153 and B3.6.4. A value of 2 for TTBCR.T1SZ sets up
/// the MMU to use TTBR1 when the top two bits of a virtual address are BOTH 1.
/// TTBR0 is used otherwise. A value of 0 for TTBCR.A1 tells the MMU that TTBR0
/// defines address space IDs. TTBCR.EAE enable extended address extensions for
/// long page descriptors.
#define TTBCR_EAE   (0x1 << 31)
#define TTBCR_A1    (0x0 << 22)
#define TTBCR_T1SZ  (0x2 << 16)
#define TTBCR_T0SZ  (0x0 << 0)
#define TTBCR_VALUE (TTBCR_EAE | TTBCR_A1 | TTBCR_T1SZ | TTBCR_T0SZ)

/// Page descriptor flags. See B3.6.1, B3.6.2, and B4.1.104.
#define MM_TYPE_PAGE_TABLE 0x3
#define MM_TYPE_PAGE       0x3
#define MM_TYPE_BLOCK      0x1
#define MM_ACCESS_FLAG     (0x1 << 10)
#define MM_ACCESS_RW       (0x0 << 15)
#define MM_ACCESS_RO       (0x1 << 15)

#define MM_DEVICE_ATTR     0x04
#define MM_NORMAL_ATTR     0x44
#define MM_MAIR0_VALUE     ((MM_DEVICE_ATTR << 8) | MM_NORMAL_ATTR)
#define MM_MAIR1_VALUE     0

#define MM_NORMAL_MAIR_IDX (0x0 << 2)
#define MM_DEVICE_MAIR_IDX (0x1 << 2)

#define MMU_NORMAL_RO_FLAGS (MM_TYPE_BLOCK | MM_ACCESS_RO | MM_NORMAL_MAIR_IDX | MM_ACCESS_FLAG)
#define MMU_NORMAL_RW_FLAGS (MM_TYPE_BLOCK | MM_ACCESS_RW | MM_NORMAL_MAIR_IDX | MM_ACCESS_FLAG)
#define MMU_DEVICE_RO_FLAGS (MM_TYPE_BLOCK | MM_ACCESS_RO | MM_DEVICE_MAIR_IDX | MM_ACCESS_FLAG)
#define MMU_DEVICE_RW_FLAGS (MM_TYPE_BLOCK | MM_ACCESS_RW | MM_DEVICE_MAIR_IDX | MM_ACCESS_FLAG)

/// 2 MiB section virtual address layout:
///
///   +----+--------+--------------------+
///   | L1 |   L2   |       Offset       |
///   +----+--------+--------------------+
///   31  30       21                    0
///
/// 4 KiB page virtual address layout:
///
///   +----+--------+--------+-----------+
///   | L1 |   L2   |   L3   |  Offset   |
///   +----+--------+--------+-----------+
///   31  30       21       12           0
#define PAGE_SHIFT         12
#define L1_TABLE_SHIFT     2
#define L2_TABLE_SHIFT     9
#define L3_TABLE_SHIFT     9
#define SECTION_SHIFT      (PAGE_SHIFT + L3_TABLE_SHIFT)
#define SECTION_SIZE       (1 << SECTION_SHIFT)
#define L1_TABLE_ENTRY_CNT (1 << L1_TABLE_SHIFT)
#define L2_TABLE_ENTRY_CNT (1 << L2_TABLE_SHIFT)
#define L3_TABLE_ENTRY_CNT (1 << L3_TABLE_SHIFT)

#define L3_SHIFT PAGE_SHIFT
#define L2_SHIFT (PAGE_SHIFT + L3_TABLE_SHIFT)
#define L1_SHIFT (L2_SHIFT + L2_TABLE_SHIFT)


/*----------------------------------------------------------------------------*/
/// Setup the TTBCR flags for the MMU.
///
/// # Returns
///
/// A TTBCR value appropriate for short page descriptors.
.global mmu_make_ttbcr_value_long
mmu_make_ttbcr_value_long:
// no fn_entry required.

  ldr     r0, =TTBCR_VALUE

// no fn_exit required.
  mov     pc, lr


/*----------------------------------------------------------------------------*/
/// Create the bootstrap kernel page tables using Large Physical Address
/// Extensions.
///
/// # Parameters
///
/// * r0 - The base of the blob.
/// * r1 - The size of the DTB or 0 if the blob is not a DTB.
///
/// # Description
///
/// Maps the kernel and, as necessary, the DTB into 2 MiB sections. The kernel
/// will re-map the pages after determining the memory layout.
///
/// The mapping will use LPAE and long page table descriptors. The bootstrap
/// should have already set the TTBR0/TTBR1 split. This code only needs to know
/// the virtual base address to choose the correct L1 table entry.
.global mmu_create_kernel_page_tables_long
mmu_create_kernel_page_tables_long:
  fn_entry
  push    {r4, r5, r6, r7, r10, r11}

  mov     r4, r0

// Align the blob size on a section.
  mov     r0, r1
  bl      section_align_size
  mov     r5, r0

// Align the size of the kernel area on a section.
  adr     r0, kernel_id_pages_end_rel
  ldr     r1, kernel_id_pages_end_rel
  add     r0, r0, r1
  bl      section_align_size
  mov     r6, r0

// Clear the page tables.
  adr     r0, kernel_pages_start_rel
  ldr     r1, kernel_pages_start_rel
  add     r0, r0, r1
  eor     r1, r1
  ldr     r2, =__kernel_pages_size
  bl      memset

  adr     r0, kernel_id_pages_start_rel
  ldr     r1, kernel_id_pages_start_rel
  add     r0, r0, r1
  eor     r1, r1
  ldr     r2, =__kernel_id_pages_size
  bl      memset

// Initialize the indirect memory attributes
  bl      init_mair

// Map the 1 GiB blocks for the identity and kernel tables and save off the base
// addresses for the level 2 identity and kernel tables.
  adr     r7, kernel_pages_start_rel
  ldr     r1, kernel_pages_start_rel
  add     r7, r7, r1

  mov     r10, r7
  ldr     r1, =__page_size
  add     r10, r10, r1

  adr     r11, kernel_id_pages_start_rel
  ldr     r1, kernel_id_pages_start_rel
  add     r11, r11, r1

  mov     r0, r7
  mov     r1, r10
  ldr     r2, =__virtual_start
  bl      init_table

  mov     r0, r7
  mov     r1, r11
  eor     r2, r2
  bl      init_table

// Map the kernel area as RW normal memory.
//
//   TODO: The code should probably be separate from the stack and page tables
//         to prevent the code from being re-written.
  mov     r0, r10
  eor     r1, r1
  ldr     r2, =__virtual_start
  add     r3, r2, r6
  sub     r3, r3, #1
  ldr     r7, =MMU_NORMAL_RW_FLAGS
  push    {r7}
  bl      map_block
  pop     {r7}

  mov     r0, r11
  eor     r1, r1
  eor     r2, r2
  add     r3, r2, r6
  sub     r3, r3, #1
  ldr     r7, =MMU_NORMAL_RW_FLAGS
  push    {r7}
  bl      map_block
  pop     {r7}

// Map the DTB area as RO normal memory. Skip this if the DTB size is zero.
// Do not need to create an identity map. The kernel will switch to virtual
// addresses before the DTB is needed.
  cmp     r5, #0
  beq     1f

  mov     r0, r10
  eor     r1, r1
  add     r1, r1, r4
  ldr     r2, =__virtual_start
  add     r2, r2, r4
  add     r3, r2, r5
  sub     r3, r3, #1
  ldr     r7, =MMU_NORMAL_RO_FLAGS
  push    {r7}
  bl      map_block
  pop     {r7}

1:
  pop     {r4, r5, r6, r7, r10, r11}
  fn_exit


/*----------------------------------------------------------------------------*/
/// Section-align the size with the next section higher.
///
/// # Parameters
///
/// * r0 - The size to align.
///
/// # Returns
///
/// The section-aligned size.
section_align_size:
// no fn_entry required.

  ldr     r1, =SECTION_SIZE
  sub     r1, r1, #1
  add     r0, r0, r1

  ldr     r1, =SECTION_SIZE
  neg     r1, r1
  and     r0, r0, r1

// no fn_exit required.
  mov     pc, lr


/*----------------------------------------------------------------------------*/
/// Initialize the indirect memory attribute registers.
init_mair:
// no fn_entry required.

  ldr     r0, =MM_MAIR0_VALUE
  mcr     p15, 0, r0, c10, c2, 0

  ldr     r0, =MM_MAIR1_VALUE
  mcr     p15, 0, r0, c10, c2, 1

// no fn_exit required.
  mov     pc, lr


/*----------------------------------------------------------------------------*/
/// Initialize a 1 GiB block.
///
/// # Parameters
///
/// * r0 - The base address of the L1 table.
/// * r1 - The base address of the L2 table.
/// * r2 - The base address of the virtual address space.
init_table:
// no fn_entry required.

// Shift the virtual address down and mask it with the entry count to get the
// entry index. Shift the entry index left by 3 since the entries are 64-bit.
  lsr     r2, r2, #L1_SHIFT
  and     r2, r2, #L1_TABLE_ENTRY_CNT - 1
  lsl     r2, r2, #3

// Create the entry. The descriptor has to be split between two 32-bit
// registers. r1 will be the lower 32-bits and the upper 32-bits will be 0. The
// table pointer is technically 28-bits (40-bits shifted down by 12-bits),
// however our physical table address is only 32-bits. The 20-bits of the
// address in r1 are the complete pointer and none of the upper attribute flags
// need to be set.
  orr     r1, r1, #MM_TYPE_PAGE_TABLE

// Store the entry in the table.
  add     r0, r0, r2
  str     r1, [r0], #4      // Lower 32-bits
  eor     r1, r1, r1        // Zero the upper 32-bits
  str     r1, [r0], #4      // Upper 32-bits

// no fn_exit required.
  mov     pc, lr


/*----------------------------------------------------------------------------*/
/// Map a block of 2 MiB sections to the L2 translation table.
///
/// # Parameters
///
/// * r0 - The base address of the L2 table.
/// * r1 - The base physical address.
/// * r2 - The base virtual address.
/// * r3 - The last virtual address.
/// * stack - The entry flags.
map_block:
// no fn_entry required.
  push {r4, r5}

  ldr     r4, [sp, #8]
  mov     r5, #L2_TABLE_ENTRY_CNT - 1

  lsr     r2, r2, #SECTION_SHIFT
  and     r2, r2, r5
  lsr     r3, r3, #SECTION_SHIFT
  and     r3, r3, r5
  lsr     r1, r1, #SECTION_SHIFT
  orr     r1, r4, r1, lsl #SECTION_SHIFT
1:
// Same as `init_table`. The table entries are 64-bit, but the 20-bit pointers
// in r1 are complete and there are no upper attributes to set. The upper 32-
// bits of the descriptor can be left as zero. Store by shifting the index left
// 3 bits.
  str     r1, [r0, r2, lsl #3]
  add     r2, r2, #1
  add     r1, r1, #SECTION_SIZE
  cmp     r2, r3
  bls     1b

  pop     {r4, r5}
// no fn_exit required.
  mov     pc, lr


.global mmu_setup_ttbr_long
mmu_setup_ttbr_long:
// no fn_entry required.

  adr     r0, kernel_pages_start_rel
  ldr     r1, kernel_pages_start_rel
  add     r0, r0, r1
  mcr     p15, 0, r0, c2, c0, 0

  eor     r0, r0
  mcr     p15, 0, r0, c2, c0, 1

// no fn_exit required.
  mov     pc, lr


.global mmu_cleanup_ttbr_long
mmu_cleanup_ttbr_long:
// no fn_entry required.

  adr     r0, kernel_pages_start_rel
  ldr     r1, kernel_pages_start_rel
  add     r0, r0, r1

  eor     r1, r1
  str     r1, [r0], #4
  str     r1, [r0], #4

// no fn_exit required.
  mov     pc, lr


/*----------------------------------------------------------------------------*/
/// See boot.S.
kernel_id_pages_start_rel:
  .word __kernel_id_pages_start - kernel_id_pages_start_rel
kernel_id_pages_end_rel:
  .word __kernel_id_pages_end - kernel_id_pages_end_rel
kernel_pages_start_rel:
  .word __kernel_pages_start - kernel_pages_start_rel
kernel_pages_end_rel:
  .word __kernel_pages_end - kernel_pages_end_rel
