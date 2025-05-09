//! ARMv7a Page Table Setup

#include "abi.h"

/// TTBCR value. See B4.1.153 and B3.6.4. A value of 0 for TTBCR.A1 tells the
/// MMU that TTBR0 defines address space IDs. TTBCR.EAE enable extended address
/// extensions for long page descriptors. TTBCR.T0SZ is 0 to let the user
/// segment fill the virtual addresses not used by the kernel segment.
/// TTBCR.T1SZ is either 1 for a 2/2 split or 2 for a 3/1 split.
#define TTBCR_EAE    (0x1 << 31)
#define TTBCR_A1     (0x0 << 22)
#define TTBCR_T1SZ_2 (0x1 << 16)
#define TTBCR_T1SZ_3 (0x2 << 16)
#define TTBCR_T0SZ   (0x0 << 0)
#define TTBCR_VALUE  (TTBCR_EAE | TTBCR_A1 | TTBCR_T0SZ)

/// Page descriptor flags. See B3.6.1, B3.6.2, and B4.1.104.
#define MM_TYPE_PAGE_TABLE 0x3
#define MM_TYPE_PAGE       0x3
#define MM_TYPE_BLOCK      0x1
#define MM_ACCESS_FLAG     (0x1 << 10)
#define MM_ACCESS_RW       (0x0 << 6)
#define MM_ACCESS_RO       (0x2 << 6)

#define MM_DEVICE_ATTR     0x04
#define MM_NORMAL_ATTR     0x44
#define MM_MAIR0_VALUE     ((MM_DEVICE_ATTR << 8) | MM_NORMAL_ATTR)
#define MM_MAIR1_VALUE     0

#define MM_NORMAL_MAIR_IDX (0x0 << 2)
#define MM_DEVICE_MAIR_IDX (0x1 << 2)

#define MMU_NORMAL_RO_BLOCK_FLAGS                                                                  \
  (MM_TYPE_BLOCK | MM_ACCESS_RO | MM_NORMAL_MAIR_IDX | MM_ACCESS_FLAG)
#define MMU_NORMAL_RW_BLOCK_FLAGS                                                                  \
  (MM_TYPE_BLOCK | MM_ACCESS_RW | MM_NORMAL_MAIR_IDX | MM_ACCESS_FLAG)
#define MMU_DEVICE_RO_BLOCK_FLAGS                                                                  \
  (MM_TYPE_BLOCK | MM_ACCESS_RO | MM_DEVICE_MAIR_IDX | MM_ACCESS_FLAG)
#define MMU_DEVICE_RW_BLOCK_FLAGS                                                                  \
  (MM_TYPE_BLOCK | MM_ACCESS_RW | MM_DEVICE_MAIR_IDX | MM_ACCESS_FLAG)

#define MMU_NORMAL_RO_PAGE_FLAGS (MM_TYPE_PAGE | MM_ACCESS_RO | MM_NORMAL_MAIR_IDX | MM_ACCESS_FLAG)
#define MMU_NORMAL_RW_PAGE_FLAGS (MM_TYPE_PAGE | MM_ACCESS_RW | MM_NORMAL_MAIR_IDX | MM_ACCESS_FLAG)
#define MMU_DEVICE_RO_PAGE_FLAGS (MM_TYPE_PAGE | MM_ACCESS_RO | MM_DEVICE_MAIR_IDX | MM_ACCESS_FLAG)
#define MMU_DEVICE_RW_PAGE_FLAGS (MM_TYPE_PAGE | MM_ACCESS_RW | MM_NORMAL_MAIR_IDX | MM_ACCESS_FLAG)

#define VEC_L2_OFFSET 0xff8
#define VEC_L3_OFFSET 0xf80

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
/// * r0 - The virtual memory split.
///
/// # Returns
///
/// Configures the bootstrap TTBCR value. If the split value is 3, a 3/1 split
/// is used. Otherwise, a 2/2 split is used.
.global mmu_make_ttbcr_value
mmu_make_ttbcr_value:
// no fn_entry required.

  ldr     r1, =TTBCR_VALUE

  cmp     r0, #3
  bne     1f

  orr     r1, #TTBCR_T1SZ_3
  b       2f

1:
  orr     r1, #TTBCR_T1SZ_2

2:
  mov     r0, r1

// no fn_exit required.
  mov     pc, lr


/*----------------------------------------------------------------------------*/
/// Create the initial kernel page tables using long descriptors.
///
/// # Parameters
///
/// * r0 - The base of the blob.
/// * r1 - The size of the DTB or 0 if the blob is not a DTB.
/// * r2 - The virtual memory split.
///
/// # Description
///
///   TODO: Break this monolith up.
///
/// Maps the kernel and, as necessary, the DTB into 2 MiB sections. The kernel
/// will re-map the pages after determining the memory layout.
///
/// The mapping will use LPAE and long page table descriptors. The start code
/// should have already set the TTBR0/TTBR1 split. This code only needs to know
/// the virtual base address to choose the correct L1 table entry.
.global mmu_create_kernel_page_tables
mmu_create_kernel_page_tables:
  fn_entry
// r4 - Scratch.
// r5 - The section-aligned kernel size.
// r6 - The saved blob base.
// r7 - The section-aligned blob size.
// r8 - The saved virtual memory split.
// r9 - Lower L2 table address.
// r10 - Upper L2 table address.
  push    {r4, r5, r6, r7, r8, r9, r10}

  mov     r6, r0
  mov     r7, r1
  mov     r8, r2

// Align the blob size on a section.
  mov     r0, r7
  bl      section_align_size
  mov     r7, r0

// Align the size of the kernel area on a section.
  adr     r0, kernel_id_pages_end_rel
  ldr     r1, kernel_id_pages_end_rel
  add     r0, r0, r1
  bl      section_align_size
  mov     r5, r0

// Initialize the indirect memory attributes
  bl      init_mair

// Clear the kernel page tables.
  adr     r0, kernel_pages_start_rel
  ldr     r1, kernel_pages_start_rel
  add     r9, r0, r1
  mov     r0, r9
  eor     r1, r1
  ldr     r2, =__kernel_pages_size
  bl      memset

// Initialize the kernel page tables. If using a 3/1 split, translation through
// TTBR1 will start at a L2 table. If using a 2/2 split, however, both TTBR0 and
// TTBR1 will start at a L1 table. Check the virtual memory split value. If 3,
// skip initializing an L1 table and use the start address as the L2 table.
  mov     r0, r9
  mov     r1, r9            // Use the same L2 table for the kernel and vectors.
  cmp     r8, #3            // Using 3/1 split?
  beq     1f                // If yes, skip L1 initialization
  ldr     r1, =__virtual_start
  bl      init_table
  
1:
  mov     r9, r0            // Save the lower L2 table address
  mov     r10, r1           // Save the upper L2 table address

// Map the vectors into the kernel page tables.
  mov     r0, r10
  adr     r1, kernel_exception_vectors_start_rel
  ldr     r3, kernel_exception_vectors_start_rel
  add     r1, r1, r3
  bl      map_vectors

// Map the kernel area as RW normal memory.
//
//   TODO: The code should probably be separate from the stack and page tables
//         to prevent the code from being re-written.
  mov     r0, r9
  eor     r1, r1
  ldr     r2, =__virtual_start
  add     r3, r2, r5
  sub     r3, r3, #1
  ldr     r4, =MMU_NORMAL_RW_BLOCK_FLAGS
  push    {r4}
  bl      map_block
  pop     {r4}

// Map the DTB area as RO normal memory. Skip this if the DTB size is zero.
// Do not need to create an identity map. The kernel will switch to virtual
// addresses before the DTB is needed.
  cmp     r7, #0
  beq     1f

  mov     r0, r9
  mov     r1, r6
  ldr     r2, =__virtual_start
  add     r2, r2, r6
  add     r3, r2, r7
  sub     r3, r3, #1
  ldr     r4, =MMU_NORMAL_RO_BLOCK_FLAGS
  push    {r4}
  bl      map_block
  pop     {r4}

1:
// Clear the kernel identity page tables.
  adr     r0, kernel_id_pages_start_rel
  ldr     r1, kernel_id_pages_start_rel
  add     r9, r0, r1
  mov     r0, r9
  eor     r1, r1
  ldr     r2, =__kernel_id_pages_size
  bl      memset

// Initialize the kernel identity page tables. The kernel identity pages are
// always going to handle more than 1 GiB since the kernel does not support a
// 1/3 split.
  mov     r0, r9
  eor     r1, r1
  bl      init_table
  mov     r9, r0            // Save the lower L2 table address
  mov     r10, r1           // Save the upper L2 table address

// Map the vectors into the kernel identity page tables.
  mov     r0, r10
  adr     r1, kernel_exception_vectors_start_rel
  ldr     r3, kernel_exception_vectors_start_rel
  add     r1, r1, r3
  bl      map_vectors

// Map the kernel area as RW normal memory. See above.
  mov     r0, r9
  eor     r1, r1
  eor     r2, r2
  add     r3, r2, r5
  sub     r3, r3, #1
  ldr     r4, =MMU_NORMAL_RW_BLOCK_FLAGS
  push    {r4}
  bl      map_block
  pop     {r4}

  pop     {r4, r5, r6, r7, r8, r9, r10}
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
/// * r0 - The base address of the L2 table for the top 1 GiB.
/// * r1 - The base address of the exception vectors.
///
/// # Description
///
///     NOTE: Assumes that the page after the L2 table is free.
///
/// Creates a L3 table with entries for the exception vector pages.
map_vectors:
// no fn_entry required.

// Get the address for the new L3 table.
  ldr     r2, =__page_size
  add     r2, r2, r0
  orr     r2, r2, #MM_TYPE_PAGE_TABLE

// Entry 511 in the L2 table covers the last 2 MiB of the address space.
  ldr     r3, =VEC_L2_OFFSET
  add     r3, r0, r3
  str     r2, [r3], #4
  eor     r2, r2
  str     r2, [r3], #4

// r3 now points to the L3 table and entry 496 covers the page at 0xffff_0000.
  mov     r0, r3
  ldr     r3, =VEC_L3_OFFSET
  add     r3, r0, r3

// Make the descriptor for the vectors.
  ldr     r2, =MMU_NORMAL_RO_PAGE_FLAGS
  orr     r1, r1, r2
  eor     r2, r2
  str     r1, [r3], #4
  str     r2, [r3], #4

// Make the descriptor for the stubs.
  ldr     r2, =__page_size
  add     r1, r1, r2
  eor     r2, r2
  str     r1, [r3], #4
  str     r2, [r3], #4

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
/// Initialize the L1 table.
///
/// # Parameters
///
/// * r0 - The base address of the L1 table.
/// * r1 - The base address of the virtual address space.
///
/// # Description
///
///   NOTE: Assumes that a 2/2 virtual memory split is being used and creates
///         entries for two L2 tables that cover the 2 GiB of the address space.
///
///   NOTE: When using a 2/2 split, the input address size is 31 bits, so only
///         bit 30 is used to index the Level 1 table. This means both Level 1
///         tables use entries 0 and 1.
///
/// # Returns
///
/// The addresses of the lower and upper L2 tables.
init_table:
// no fn_entry required.

// Get the entry address.
  lsr     r2, r1, #L1_SHIFT // Top two bits are the index
  and     r2, r2, #1        // See note in function header
  lsl     r2, r2, #3        // 8 bytes per entry
  add     r2, r0, r2        // Add the base address

// Create the table entry for the lower 1 GiB. The descriptor has to be split
// between two 32-bit registers. r3 will be the lower 32-bits and the upper
// 32-bits will be 0 since our physical address does not need the extra 8 bits
// and we do not need to set any of the upper attributes.
  ldr     r3, =__page_size
  add     r0, r0, r3        // r0 is now the lower L2 table address
  mov     r3, r0
  orr     r3, r3, #MM_TYPE_PAGE_TABLE

// Store the entry in the table.
  str     r3, [r2], #4      // Lower 32-bits
  eor     r3, r3
  str     r3, [r2], #4      // Upper 32-bits

// Create the table entry for the upper 1 GiB. See above.
  ldr     r3, =__page_size
  add     r1, r0, r3        // r1 is now the upper L2 table address
  mov     r3, r1
  orr     r3, r3, #MM_TYPE_PAGE_TABLE
  
// Store the entry in the table.
  str     r3, [r2], #4      // Lower 32-bits
  eor     r3, r3
  str     r3, [r2], #4      // Upper 32-bits

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


/*----------------------------------------------------------------------------*/
/// Setup the translation table registers before enabling the MMU.
///
/// # Description
///
/// Sets up TTBR0 with the identity tables and TTBR1 with the kernel tables.
.global mmu_setup_ttbr
mmu_setup_ttbr:
// no fn_entry required.

// Set TTBR1 to the kernel pages.
  adr     r0, kernel_pages_start_rel
  ldr     r1, kernel_pages_start_rel
  add     r0, r0, r1
  eor     r1, r1
  mcrr    p15, 1, r0, r1, c2

// Set TTBR0 to the identity pages.
  adr     r0, kernel_id_pages_start_rel
  ldr     r1, kernel_id_pages_start_rel
  add     r0, r0, r1
  eor     r1, r1
  mcrr    p15, 0, r0, r1, c2

// no fn_exit required.
  mov     pc, lr


/*----------------------------------------------------------------------------*/
/// Cleanup the translation table registers after enabling the MMU.
///
/// # Description
///
/// Zeros out TTBR0 leaving TTBR1 with the kernel pages.
.global mmu_cleanup_ttbr
mmu_cleanup_ttbr:
// no fn_entry required.

// Zero out TTBR0.
  eor     r0, r0
  eor     r1, r1
  mcrr    p15, 0, r0, r1, c2

// no fn_exit required.
  mov     pc, lr


/*----------------------------------------------------------------------------*/
/// See start.S.
///
///   TODO: These should probably be passed in as parameters rather than
///         re-defining them.
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
