//! ARMv7a Page Table Bootstrap without Large Physical Address Extensions

#include "abi.h"

// TTBCR value. See B4.1.153 and B3.6.4. A value of 1 disables extended
// addresses, enables TTBR1 translation, and enables TTBR0 translation. If the
// top bit of the virtual address is 0, TTBR0 is used. Otherwise, TTBR1 is used.
#define TTBCR_VALUE 0x1

// Page descriptor flags. See B3.5.1. The Access Flag Enable bit should be set
// to 1 in SCTLR to use the 2-bit access control model where AP[2] (bit 15)
// disables writing and A[1] (bit 11) enables unpriviledged access. AP[0] (bit
// 10) is the Access Flag.
#define MM_TYPE_PAGE_TABLE 0x1
#define MM_TYPE_PAGE       0x2
#define MM_TYPE_BLOCK      0x2
#define MM_L1_ACCESS_FLAG  (0b1 << 10)
#define MM_L1_ACCESS_RW    (0b0 << 15)
#define MM_L1_ACCESS_RO    (0b1 << 15)
#define MM_L2_ACCESS_FLAG  (0b1 << 4)
#define MM_L2_ACCESS_RW    (0b0 << 9)
#define MM_L2_ACCESS_RO    (0b1 << 9)
#define MM_DEVICE_CB       (0b01 << 2)
#define MM_NORMAL_CB       (0b10 << 2)

#define MMU_PAGE_PAGE_TABLE_FLAGS MM_TYPE_PAGE_TABLE

#define MMU_L1_NORMAL_RO_BLOCK_FLAGS                                                               \
  (MM_TYPE_BLOCK | MM_L1_ACCESS_RO | MM_NORMAL_CB | MM_L1_ACCESS_FLAG)
#define MMU_L1_NORMAL_RW_BLOCK_FLAGS                                                               \
  (MM_TYPE_BLOCK | MM_L1_ACCESS_RW | MM_NORMAL_CB | MM_L1_ACCESS_FLAG)
#define MMU_L1_DEVICE_RO_BLOCK_FLAGS                                                               \
  (MM_TYPE_BLOCK | MM_L1_ACCESS_RO | MM_DEVICE_CB | MM_L1_ACCESS_FLAG)
#define MMU_L1_DEVICE_RW_BLOCK_FLAGS                                                               \
  (MM_TYPE_BLOCK | MM_L1_ACCESS_RW | MM_DEVICE_CB | MM_L1_ACCESS_FLAG)

#define MMU_L2_NORMAL_RO_PAGE_FLAGS                                                                \
  (MM_TYPE_PAGE | MM_L2_ACCESS_RO | MM_NORMAL_CB | MM_L2_ACCESS_FLAG)
#define MMU_L2_NORMAL_RW_PAGE_FLAGS                                                                \
  (MM_TYPE_PAGE | MM_L2_ACCESS_RW | MM_NORMAL_CB | MM_L2_ACCESS_FLAG)
#define MMU_L2_DEVICE_RO_PAGE_FLAGS                                                                \
  (MM_TYPE_PAGE | MM_L2_ACCESS_RO | MM_DEVICE_CB | MM_L2_ACCESS_FLAG)
#define MMU_L2_DEVICE_RW_PAGE_FLAGS                                                                \
  (MM_TYPE_PAGE | MM_L2_ACCESS_RW | MM_DEVICE_CB | MM_L2_ACCESS_FLAG)

/// 1 MiB section virtual address layout:
///
///   +-+----------+-------------------+
///   |/|    L1    |       Offset      |
///   +-+----------+-------------------+
///    31         20                   0
///
/// 4 KiB page virtual address layout:
///
///   +-+----------+-------+-----------+
///   |/|    L1    |  L2   |  Offset   |
///   +-+----------+-------+-----------+
///    31         20      12           0
#define PAGE_SHIFT         12
#define L1_TABLE_SHIFT     12
#define L2_TABLE_SHIFT     8
#define SECTION_SHIFT      (PAGE_SHIFT + L2_TABLE_SHIFT)
#define SECTION_SIZE       (1 << SECTION_SHIFT)
#define L1_TABLE_ENTRY_CNT (1 << L1_TABLE_SHIFT)
#define L2_TABLE_ENTRY_CNT (1 << L2_TABLE_SHIFT)
#define L1_TABLE_SIZE      (L1_TABLE_ENTRY_CNT << 2)
#define L2_TABLE_SIZE      (L2_TABLE_ENTRY_CNT << 2)

#define L2_SHIFT PAGE_SHIFT
#deifne L1_SHIFT (L2_SHIFT + L2_TABLE_SHIFT)


/*----------------------------------------------------------------------------*/
/// Setup the TTBCR flags for the MMU.
///
/// # Returns
///
/// A TTBCR value appropriate for short page descriptors.
.global mmu_make_ttbcr_value_short
mmu_make_ttbcr_value_short:
  // no fn_entry required.

  ldr     r0, =TTBCR_VALUE

  // no fn_exit required.
  mov     pc, lr


/*----------------------------------------------------------------------------*/
/// Create the bootstrap kernel page tables using short descriptors.
///
/// # Parameters
///
/// * r0 - The base of the blob.
/// * r1 - The size of the DTB or 0 if the blob is not a DTB.
///
/// # Description
///
/// Maps the kernel and, as necessary, the DTB into 1 MiB sections. The kernel
/// will re-map the pages after determining the memory layout.
.global mmu_create_kernel_page_tables_short
mmu_create_kernel_page_tables_short:
  fn_entry
  push    {r4, r5, r6, r7, r10, r11}

  mov     r4, r0

// Align the blob size on a section.
  mov     r0, r1
  bl      section_align_size
  mov     r5, r0

// Calculate the starting addresses of the page tables.
  adr     r10, kernel_pages_start_rel
  ldr     r1, kernel_pages_start_rel
  add     r10, r10, r1

  adr     r11, kernel_id_pages_start_rel
  ldr     r1, kernel_id_pages_start_rel
  add     r11, r11, r1

// Align the size of the kernel area on a section.
  adr     r0, kernel_id_pages_end_rel
  ldr     r1, kernel_id_pages_end_rel
  add     r0, r0, r1
  bl      section_align_size
  mov     r6, r0

// Clear the page tables.
  mov     r0, r10
  mov     r1, #0
  ldr     r2, =__kernel_pages_size
  bl      memset

  mov     r0, r11
  mov     r1, #0
  ldr     r2, =__kernel_id_pages_size
  bl      memset

// Map the vectors.
  adr     r7, kernel_exception_vectors_start_rel
  ldr     r1, kernel_exception_vectors_start_rel
  add     r7, r7, r1

  mov     r0, r10
  mov     r1, r0
  add     r1, r1, #L1_TABLE_SIZE
  mov     r2, r7
  ldr     r3, =__virtual_start
  bl      map_vectors

  mov     r0, r11
  mov     r1, r0
  add     r1, r1, #L1_TABLE_SIZE
  mov     r2, r7
  mov     r3, #0
  bl      map_vectors

// Map the kernel area as RW normal memory.
//
//   TODO: The code should probably be separate from the stack and page tables
//         to prevent the code from being re-written.
  mov     r0, r10
  mov     r1, #0
  ldr     r2, =__virtual_start
  add     r3, r2, r6
  sub     r3, r3, #1
  ldr     r7, =MMU_L1_NORMAL_RW_BLOCK_FLAGS
  push    {r7}
  bl      map_block
  pop     {r7}

  mov     r0, r11
  mov     r1, #0
  mov     r2, #0
  add     r3, r2, r6
  sub     r3, r3, #1
  ldr     r7, =MMU_L1_NORMAL_RW_BLOCK_FLAGS
  push    {r7}
  bl      map_block
  pop     {r7}

// Map the DTB area as RO normal memory. Skip this if the DTB size is zero.
// Do not need to create an identity map. The kernel will switch to virtual
// addresses before the DTB is needed.
  cmp     r5, #0
  beq     skip_dtb_mapping

  mov     r0, r10
  mov     r1, #0
  add     r1, r1, r4
  ldr     r2, =__virtual_start
  add     r2, r2, r4
  add     r3, r2, r5
  sub     r3, r3, #1
  ldr     r7, =MMU_L1_NORMAL_RO_BLOCK_FLAGS
  push    {r7}
  bl      map_block
  pop     {r7}

skip_dtb_mapping:
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
/// Map the exception vector page.
///
/// # Parameters
///
/// * r0 - The base address of the L1 table.
/// * r1 - The base address of the L2 table for the top 1 MiB.
/// * r2 - The base address of the exception vectors.
map_vectors:
  // no fn_entry required.
  push    {r4, r5}

  // Entry 4095 is the last entry in the L1 table and covers the top 1 MiB of
  // the virtual address space.
  ldr     r4, =0x3ffc

  // Write the L1 descriptor. The L2 table address is assumed to be aligned on a
  // 1 KiB boundary.
  ldr     r5, =MMU_PAGE_PAGE_TABLE_FLAGS
  orr     r5, r5, r1
  str     r5, [r0, r4]

  // Entry 240 covers the page at 0xffff_0000.
  ldr     r4, =0x3c0

  // Write the vectors L2 descriptor. The exception vectors are assumed to be
  // aligned on a page.
  ldr     r5, =MMU_L2_NORMAL_RO_PAGE_FLAGS
  orr     r5, r5, r2
  str     r5, [r1, r4]

  // Write the stubs L2 descriptor.
  ldr     r5, =MMU_L2_NORMAL_RO_PAGE_FLAGS
  ldr     r3, =__page_size
  add     r2, r2, r3
  orr     r5, r5, r2
  add     r4, r4, #4
  str     r5, [r1, r4]

  pop     {r4, r5}
  // no fn_exit required.
  mov     pc, lr


/*----------------------------------------------------------------------------*/
/// Map a block of 1 MiB sections to the L1 translation table.
///
/// # Parameters
///
/// * r0 - The base address of the L1 table.
/// * r1 - The base physical address.
/// * r2 - The base virtual address.
/// * r3 - The last virtual address.
/// * stack - The entry flags.
map_block:
  // no fn_entry required.
  push {r4, r5}

  ldr     r4, [sp, #8]
  mov     r5, #L1_TABLE_ENTRY_CNT - 1

  lsr     r2, r2, #SECTION_SHIFT
  and     r2, r2, r5
  lsr     r3, r3, #SECTION_SHIFT
  and     r3, r3, r5
  lsr     r1, r1, #SECTION_SHIFT
  orr     r1, r4, r1, lsl #SECTION_SHIFT
map_block_loop:
  str     r1, [r0, r2, lsl #2]
  add     r2, r2, #1
  add     r1, r1, #SECTION_SIZE
  cmp     r2, r3
  bls     map_block_loop

  pop     {r4, r5}
  // no fn_exit required.
  mov     pc, lr


/*----------------------------------------------------------------------------*/
/// See boot.S.
kernel_exception_vectors_start_rel:
  .word __kernel_exception_vectors_start - kernel_exception_vectors_start_rel
kernel_id_pages_start_rel:
  .word __kernel_id_pages_start - kernel_id_pages_start_rel
kernel_id_pages_end_rel:
  .word __kernel_id_pages_end - kernel_id_pages_end_rel
kernel_pages_start_rel:
  .word __kernel_pages_start - kernel_pages_start_rel
kernel_pages_end_rel:
  .word __kernel_pages_end - kernel_pages_end_rel
