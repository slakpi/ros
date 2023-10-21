//! ARMv7a Page Table Bootstrap

#include "abi.h"
#include "mmu.h"

/// 1 MiB section virtual address layout:
///
///   +----------------+--------------------+
///   |       L1       |       Offset       |
///   +----------------+--------------------+
///   31              20                    0
///
/// 4 KiB page virtual address layout:
///
///   +----------------+--------+-----------+
///   |       L1       |   L2   |  Offset   |
///   +----------------+--------+-----------+
///   31              20       12           0
#define PAGE_SHIFT         12
#define L1_TABLE_SHIFT     12
#define L2_TABLE_SHIFT     8
#define SECTION_SHIFT      (PAGE_SHIFT + L2_TABLE_SHIFT)
#define SECTION_SIZE       (1 << SECTION_SHIFT)
#define L1_TABLE_ENTRY_CNT (1 << L1_TABLE_SHIFT)
#define L2_TABLE_ENTRY_CNT (1 << L2_TABLE_SHIFT)

#define L2_SHIFT PAGE_SHIFT
#deifne L1_SHIFT (L2_SHIFT + L2_TABLE_SHIFT)


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
/// Maps the kernel and, as necessary, the DTB into 1 MiB sections. The kernel
/// will re-map the pages after determining the memory layout.
.global create_kernel_pages
create_kernel_pages:
  fn_entry
  push    {r4, r5, r6, r7}

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

// Map the kernel area as RW normal memory.
// TODO: The code should probably be separate from the stack and page tables to
//       prevent the code from being re-written.
  adr     r0, kernel_pages_start_rel
  ldr     r1, kernel_pages_start_rel
  add     r0, r0, r1
  mov     r1, #0
  ldr     r2, =__virtual_start
  add     r3, r2, r6
  sub     r3, r3, #1
  ldr     r7, =MMU_NORMAL_RW_FLAGS
  push    {r7}
  bl      map_block
  pop     {r7}

  adr     r0, kernel_id_pages_start_rel
  ldr     r1, kernel_id_pages_start_rel
  add     r0, r0, r1
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

  adr     r0, kernel_pages_start_rel
  ldr     r1, kernel_pages_start_rel
  add     r0, r0, r1
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
  pop     {r4, r5, r6, r7}
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
kernel_id_pages_start_rel:
  .word __kernel_id_pages_start - kernel_id_pages_start_rel
kernel_id_pages_end_rel:
  .word __kernel_id_pages_end - kernel_id_pages_end_rel
kernel_pages_start_rel:
  .word __kernel_pages_start - kernel_pages_start_rel
kernel_pages_end_rel:
  .word __kernel_pages_end - kernel_pages_end_rel
