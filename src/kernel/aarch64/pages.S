#include "abi.h"
#include "mmu.h"
#include "peripherals.h"

#define PHYS_START      0x0
#define VIRT_START      0x0
// #define VIRT_START      0xffff000000000000
#define PAGE_SHIFT      12
#define TABLE_SHIFT     9
#define SECTION_SHIFT   (PAGE_SHIFT + TABLE_SHIFT)
#define SECTION_SIZE    (1 << SECTION_SHIFT)	
#define TABLE_ENTRY_CNT (1 << TABLE_SHIFT)

#define PGD_SHIFT (PAGE_SHIFT + (3 * TABLE_SHIFT))
#define PUD_SHIFT (PAGE_SHIFT + (2 * TABLE_SHIFT))
#define PMD_SHIFT (PAGE_SHIFT + TABLE_SHIFT)


.macro create_pgd_entry, tbl, virt, tmp1, tmp2
  create_table_entry \tbl, \virt, PGD_SHIFT, \tmp1, \tmp2
  create_table_entry \tbl, \virt, PUD_SHIFT, \tmp1, \tmp2
.endm


.macro create_table_entry, tbl, virt, shift, tmp1, tmp2
  lsr     \tmp1, \virt, #\shift
  and     \tmp1, \tmp1, #TABLE_ENTRY_CNT - 1
  ldr     \tmp2, =__page_size
  add     \tmp2, \tbl, \tmp2
  orr     \tmp2, \tmp2, #MM_TYPE_PAGE_TABLE	
  str     \tmp2, [\tbl, \tmp1, lsl #3]
  ldr     \tmp2, =__page_size
  add     \tbl, \tbl, \tmp2
.endm


.macro create_block_map, tbl, phys, start, end, flags, tmp1
  lsr     \start, \start, #SECTION_SHIFT
  and     \start, \start, #TABLE_ENTRY_CNT - 1
  lsr     \end, \end, #SECTION_SHIFT
  and     \end, \end, #TABLE_ENTRY_CNT - 1
  lsr     \phys, \phys, #SECTION_SHIFT
  mov     \tmp1, #\flags
  orr     \phys, \tmp1, \phys, lsl #SECTION_SHIFT
9999:
  str     \phys, [\tbl, \start, lsl #3]
  add     \start, \start, #1
  add     \phys, \phys, #SECTION_SIZE
  cmp     \start, \end
  b.ls    9999b
.endm


///-------------------------------------------------------------------------------------------------
/// @fn create_kernel_pages(x0, x1)
/// @brief   Create the bootstrap kernel pages.
/// @details Maps the kernel and, as necessary, the DTB into 2 MiB sections. The
///          kernel will re-map the pages after determining the memory layout.
/// @param[in] x0 The base of the blob.
/// @param[in] x1 The size of the DTB or 0 if the blob is not a DTB.
.global create_kernel_pages
create_kernel_pages:
  fn_entry
  stp     x19, x20, [sp, #-16]!
  stp     x21, x22, [sp, #-16]!

/*------------------------------------------------------------------------------
Align the blob size on a section.
------------------------------------------------------------------------------*/
  mov     x19, x0
  mov     x0, x1
  bl      section_align_size
  mov     x20, x0

/*------------------------------------------------------------------------------
Align the size of the kernel area on a section.
------------------------------------------------------------------------------*/
  ldr     x0, =__kernel_pages_end
  bl      section_align_size
  mov     x21, x0

/*------------------------------------------------------------------------------
Clear the page tables.
------------------------------------------------------------------------------*/
  ldr     x0, =__kernel_pages_start
  mov     x1, #0
  ldr     x2, =__kernel_pages_size
  bl      memset

/*------------------------------------------------------------------------------
Create the first- and second-level page tables.
------------------------------------------------------------------------------*/
  ldr     x9, =__kernel_pages_start
  ldr     x11, =VIRT_START
  create_pgd_entry x9, x11, x12, x13

/*------------------------------------------------------------------------------
Map the kernel area. Do not reset x9, it has been incremented to the start of
the third-level table.
------------------------------------------------------------------------------*/
  ldr     x10, =PHYS_START
  ldr     x11, =VIRT_START
  create_block_map x9, x10, x11, x21, MMU_FLAGS, x12

/*------------------------------------------------------------------------------
Map the DTB area.
------------------------------------------------------------------------------*/
  ldr     x11, =VIRT_START
  add     x11, x11, x19
  create_block_map x9, x19, x11, x20, MMU_FLAGS, x12

/*------------------------------------------------------------------------------
Map the peripheral area.
------------------------------------------------------------------------------*/
  ldr     x10, =PERIPHERAL_BASE
  ldr     x11, =(VIRT_START + PERIPHERAL_BASE)
  ldr     x12, =(VIRT_START + PERIPHERAL_BASE + PERIPHERAL_BLOCK_SIZE - 1)
  create_block_map x9, x10, x11, x12, MMU_FLAGS, x13

  ldp     x21, x22, [sp], #16
  ldp     x19, x20, [sp], #16
  fn_exit
  ret


///-------------------------------------------------------------------------------------------------
/// @fn section_align_size(x0)
/// @brief   Section-align the size with the next section higher.
/// @param[in] x0 The size to align.
/// @returns The section-aligned size.
section_align_size:
  // no fn_entry required.

  mov     x9, #SECTION_SIZE
  sub     x10, x9, 1
  add     x0, x0, x10

  neg     x9, x9
  and     x0, x0, x9

  // no fn_exit required.
  ret
