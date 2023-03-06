#include "abi.h"
#include "mmu.h"
#include "rpi_peripherals.h"

#define PAGE_SHIFT      12
#define TABLE_SHIFT     9
#define SECTION_SHIFT   (PAGE_SHIFT + TABLE_SHIFT)
#define SECTION_SIZE    (1 << SECTION_SHIFT)	
#define TABLE_ENTRY_CNT (1 << TABLE_SHIFT)

#define L1_SHIFT (PAGE_SHIFT + (3 * TABLE_SHIFT))
#define L2_SHIFT (PAGE_SHIFT + (2 * TABLE_SHIFT))


///-------------------------------------------------------------------------------------------------
/// @fn create_kernel_pages
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
  stp     x23, x24, [sp, #-16]!

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
  adrp    x0, __kernel_id_pages_end
  bl      section_align_size
  mov     x21, x0

/*------------------------------------------------------------------------------
Clear the page tables.
------------------------------------------------------------------------------*/
  adrp    x0, __kernel_pages_start
  mov     x1, #0
  ldr     x2, =__kernel_pages_size
  bl      memset

  adrp    x0, __kernel_id_pages_start
  mov     x1, #0
  ldr     x2, =__kernel_id_pages_size
  bl      memset

/*------------------------------------------------------------------------------
Create the first- and second-level page tables.
------------------------------------------------------------------------------*/
  adrp    x0, __kernel_pages_start
  ldr     x1, =__virtual_start
  bl      init_tables
  mov     x22, x0

  adrp    x0, __kernel_id_pages_start
  mov     x1, #0
  bl      init_tables
  mov     x23, x0

/*------------------------------------------------------------------------------
Map the kernel area as RW normal memory.
------------------------------------------------------------------------------*/
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

/*------------------------------------------------------------------------------
Map the DTB area as RO normal memory. Skip this if the DTB size is zero. Do not
need to create an identity map. The kernel will switch to virtual addresses
before the DTB is needed.
------------------------------------------------------------------------------*/
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


///-------------------------------------------------------------------------------------------------
/// @fn section_align_size
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


///-------------------------------------------------------------------------------------------------
/// @fn init_tables
/// @brief   Create first- and second-level page tables for the first 1 GiB of
///          the virtual address space.
/// @param[in] x0 The base address of the L4 table.
/// @param[in] x1 The base address of the virtual address space.
/// @returns Returns the base address of the third-level page table.
init_tables:
  fn_entry

  mov     x2, #L1_SHIFT
  bl      create_table_entry

  mov     x2, #L2_SHIFT
  bl      create_table_entry

  fn_exit
  ret


///-------------------------------------------------------------------------------------------------
/// @fn create_table_entry
/// @brief Helper for @a init_tables. Do not call directly.
/// @param[in] x0 The base address of the L4 table.
/// @param[in] x1 The base address of the virtual address space.
/// @param[in] x2 The shift specifying the L4 or L3 table.
create_table_entry:
  // no fn_entry required.

  lsr     x9, x1, x2
  and     x9, x9, #TABLE_ENTRY_CNT - 1
  ldr     x10, =__page_size
  add     x10, x0, x10
  orr     x10, x10, #MM_TYPE_PAGE_TABLE	
  str     x10, [x0, x9, lsl #3]
  ldr     x10, =__page_size
  add     x0, x0, x10

  // no fn_exit required.
  ret


///-------------------------------------------------------------------------------------------------
/// @fn map_block
/// @brief Map a block of addresses within the first 1 GiB of the physical
///        address space into the virtual address space.
/// @param[in] x0 The base address of the L2 table.
/// @param[in] x1 The base physical address.
/// @param[in] x2 The base virtual address.
/// @param[in] x3 The last virtual address.
/// @param[in] x4 The entry flags.
map_block:
  // no fn_entry required.

  lsr     x2, x2, #SECTION_SHIFT
  and     x2, x2, #TABLE_ENTRY_CNT - 1
  lsr     x3, x3, #SECTION_SHIFT
  and     x3, x3, #TABLE_ENTRY_CNT - 1
  lsr     x1, x1, #SECTION_SHIFT
  orr     x1, x4, x1, lsl #SECTION_SHIFT
map_block_loop:
  str     x1, [x0, x2, lsl #3]
  add     x2, x2, #1
  add     x1, x1, #SECTION_SIZE
  cmp     x2, x3
  b.ls    map_block_loop

  // no fn_exit required.
  ret
