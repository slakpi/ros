//! ARMv7a Page Table Bootstrap
//!
//!   NOTE: Assumes that the kernel is running on a CPU that supports Large
//!         Physical Address Extensions and long page table descriptors.

#include "abi.h"
#include "mmu.h"

/// 2 MiB section virtual address layout with long page table descriptors:
///
///   +----+----------+---------------------+
///   | L1 |    L2    |       Offset        |
///   +----+----------+---------------------+
///   31  30         21                     0
///
/// 4 KiB page virtual address layout with long page table descriptors:
///
///   +----+----------+----------+----------+
///   | L1 |    L2    |    L3    |  Offset  |
///   +----+----------+----------+----------+
///   31  30         21         12          0
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
/// Create the bootstrap kernel pages.
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
.global mmu_create_kernel_pages
mmu_create_kernel_pages:
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
  mov     r1, #0
  ldr     r2, =__kernel_pages_size
  bl      memset

  adr     r0, kernel_id_pages_start_rel
  ldr     r1, kernel_id_pages_start_rel
  add     r0, r0, r1
  mov     r1, #0
  ldr     r2, =__kernel_id_pages_size
  bl      memset

// Create the L1 tables and save the L2 table pointers.
  adr     r0, kernel_pages_start_rel
  ldr     r1, kernel_pages_start_rel
  add     r0, r0, r1
  ldr     r1, =__virtual_start
  bl      init_table
  mov     r10, r0

  adr     r0, kernel_id_pages_start_rel
  ldr     r1, kernel_id_pages_start_rel
  add     r0, r0, r1
  mov     r1, #0
  bl      init_table
  mov     r11, r0

// Map the kernel area as RW normal memory.
//
//   TODO: The code should probably be separate from the stack and page tables
//         to prevent the code from being re-written.
  mov     r0, r10
  mov     r1, #0
  ldr     r2, =__virtual_start
  add     r3, r2, r6
  sub     r3, r3, #1
  ldr     r7, =MMU_NORMAL_RW_FLAGS
  push    {r7}
  bl      map_block
  pop     {r7}

  mov     r0, r11
  mov     r1, #0
  mov     r2, #0
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
  beq     skip_dtb_mapping

  mov     r0, r10
  mov     r1, #0
  add     r1, r1, r4
  ldr     r2, =__virtual_start
  add     r2, r2, r4
  add     r3, r2, r5
  sub     r3, r3, #1
  ldr     r7, =MMU_NORMAL_RO_FLAGS
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
/// Initialize the L1 page table for the first 1 GiB of the physical address
/// space.
///
/// # Parameters
///
/// * r0 - The base address of the L1.
/// * r1 - The base address of the virtual address space.
///
/// # Returns
///
/// The address of the next page after the table.
init_table:
  // no fn_entry required.
  push    {r4, r5, r6, r7}

  // Shift the virtual address down and mask it with the entry count to get the
  // entry index. Shift the entry index left by 3 since the entries are 64-bit.
  ldr     r5, =L1_SHIFT
  lsr     r4, r1, r5
  and     r4, r4, #L1_TABLE_ENTRY_CNT - 1
  lsl     r4, r4, #3

  // Setup the indirect memory attributes for the L1 entry. Use the top bit of
  // the virtual address to choose MAIR0 or MAIR1. Write the attributes for the
  // entry to the first 8 bits.
  mov     r5, r1
  lsr     r5, r5, #31
  and     r5, r5, #1

  ldr     r7, =0xffffff00

  cmp     r5, #1
  bne     set_mair0

  mrc     p15, 0, r6, c10, c2, 1
  and     r6, r6, r7
  orr     r6, r6, #MM_L1_NORMAL_ATTR
  mcr     p15, 0, r6, c10, c2, 0
  b       set_descriptor

set_mair0:
  mrc     p15, 0, r6, c10, c2, 0
  and     r6, r6, r7
  orr     r6, r6, #MM_L1_NORMAL_ATTR
  mcr     p15, 0, r6, c10, c2, 0

set_descriptor:
  // Save off the MAIR index. The index is three bits where the top bit selects
  // MAIR0 or MAIR1. The bottom two bits will be zero since we are using the
  // first nibble. The shift is 4 bits since the index starts at bit 2 in the
  // descriptor.
  mov     r7, r5
  lsl     r7, #4

  // Get the pointer to the table at the next page. Assume the address is page-
  // aligned, so the offset bits are already zero.
  ldr     r5, =__page_size
  add     r5, r0, r5  

  // Create the entry. The descriptor has to be split between two 32-bit
  // registers. r5 will be the lower 32-bits and r6 will be the upper 32-bits.
  // The table pointer is technically 28-bits (40-bits shifted down by 12-bits)
  // and split between the two registers (bits 12-31 in r5 and bits 0-7 in
  // r6). However, our physical table address is only 32-bits, so the 20-bits
  // of the address in r5 are the complete pointer.
  orr     r5, r5, #MM_TYPE_PAGE_TABLE
  orr     r5, r5, r7
  mov     r6, #0

  // Store the entry in the table.
  str     r5, [r0, r4]
  add     r4, r4, #4
  str     r6, [r0, r4]

  // Return the address of the next page table.
  ldr     r5, =__page_size
  add     r0, r0, r5

  pop     {r4, r5, r6, r7}
  // no fn_exit required.
  mov     pc, lr


/*----------------------------------------------------------------------------*/
/// Map a block of 2 MiB sections to the L2 translation table.
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
kernel_id_pages_start_rel:
  .word __kernel_id_pages_start - kernel_id_pages_start_rel
kernel_id_pages_end_rel:
  .word __kernel_id_pages_end - kernel_id_pages_end_rel
kernel_pages_start_rel:
  .word __kernel_pages_start - kernel_pages_start_rel
kernel_pages_end_rel:
  .word __kernel_pages_end - kernel_pages_end_rel
