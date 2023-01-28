#include "abi.h"
#include "mmu.h"

// #define VA_START        0xffff000000000000
#define VA_START        0x0
#define PAGE_MASK       0xfffffffffffff000
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
/// @details Maps the kernel and, as necessary, the DTB. The kernel will create
///          additional pages after parsing the memory configuration.
/// @param[in] x0 The base of the blob.
/// @param[in] x1 The size of the DTB or 0 if the blob is not a DTB.
.global create_kernel_pages
create_kernel_pages:
  fn_entry

  mov     x19, x0
  mov     x0, x1
  bl      section_align_size
  mov     x20, x0

  ldr     x0, =__kernel_pages_start
  mov     x1, #0
  ldr     x2, =__kernel_pages_size
  bl      memset

  ldr     x9, =__kernel_pages_start
  ldr     x10, =__kernel_start
  create_pgd_entry x9, x10, x11, x12

  ldr     x0, =__kernel_size
  bl      section_align_size
  mov     x21, x0
  add     x22, x21, x10

  mov     x11, xzr
  mov     x12, #VA_START
  create_block_map x0, x11, x12, x22, MMU_FLAGS, x13

  fn_exit
  ret


///-------------------------------------------------------------------------------------------------
/// @fn page_align_size(x0)
/// @brief   Page-align the size with the next page higher.
/// @param[in] x0 The size to align.
/// @returns The paged-aligned size.
page_align_size:
  // no fn_entry required.

  ldr     x9, =__page_size
  sub     x9, x9, #1
  add     x0, x0, x9

  add     x9, x9, #1
  neg     x9, x9
  and     x0, x0, x9

  // no fn_exit required.
  ret


///-------------------------------------------------------------------------------------------------
/// @fn section_align_size(x0)
/// @brief   Section-align the size with the next section higher.
/// @param[in] x0 The size to align.
/// @returns The section-aligned size.
section_align_size:
  // no fn_entry required.

  mov     x9, #SECTION_SIZE - 1
  add     x0, x0, x9

  mov     x9, #SECTION_SIZE
  neg     x9, x9
  and     x0, x0, x9

  // no fn_exit required.
  ret
