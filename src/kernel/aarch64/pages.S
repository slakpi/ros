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
  ldr     x0, =__kernel_pages_start
  ldr     x1, =VIRT_START
  bl      init_tables
  mov     x22, x0

/*------------------------------------------------------------------------------
Map the kernel area.
------------------------------------------------------------------------------*/
  mov     x0, x22
  ldr     x1, =PHYS_START
  ldr     x2, =VIRT_START
  mov     x3, x21
  sub     x3, x3, 1
  mov     x4, #MMU_FLAGS
  bl      map_block

/*------------------------------------------------------------------------------
Map the DTB area.
------------------------------------------------------------------------------*/
  mov     x0, x22
  ldr     x1, =PHYS_START
  add     x1, x1, x19
  ldr     x2, =VIRT_START
  add     x2, x2, x19
  add     x3, x2, x20
  sub     x3, x3, 1
  mov     x4, #MMU_FLAGS
  bl      map_block

/*------------------------------------------------------------------------------
Map the peripheral area.
------------------------------------------------------------------------------*/
  mov     x0, x22
  ldr     x1, =PERIPHERAL_BASE
  ldr     x2, =(VIRT_START + PERIPHERAL_BASE)
  ldr     x3, =(VIRT_START + PERIPHERAL_BASE + PERIPHERAL_BLOCK_SIZE - 1)
  mov     x4, #MMU_FLAGS
  bl      map_block

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


///-------------------------------------------------------------------------------------------------
/// @fn init_tables(x0, x1)
/// @brief Create first- and second-level page tables for the first 1 GiB of the
///        virtual address space.
/// @param[in] x0 The base address of the PGD table.
/// @param[in] x1 The base address of the virtual address space.
init_tables:
  fn_entry
  stp     x19, x20, [sp, #16]!

  mov     x19, x0
  mov     x20, x1

  mov     x2, #PGD_SHIFT
  bl      create_table_entry

  mov     x1, x20
  mov     x2, #PUD_SHIFT
  bl      create_table_entry

  ldp     x19, x20, [sp], #16
  fn_exit
  ret


///-------------------------------------------------------------------------------------------------
/// @fn create_table_entry(x0, x1, x2)
/// @brief Helper for @a init_tables. Do not call directly.
/// @param[in] x0 The base address of the PGD table.
/// @param[in] x1 The base address of the virtual address space.
/// @param[in] x2 The shift specifying the PGD or PUD table.
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
/// @fn map_block(x0, x1, x2, x3, x4)
/// @brief Map a block of addresses within the first 1 GiB of the physical
///        address space into the virtual address space.
/// @param[in] tbl   The base address of the PMD table.
/// @param[in] phys  The base physical address.
/// @param[in] start The base virtual address.
/// @param[in] end   The last virtual address.
/// @param[in] flags The entry flags.
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
