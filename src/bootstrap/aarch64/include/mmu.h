//! AArch64 MMU Configuration

#if !defined MMU_H
#define MMU_H

// EL1 translation control register configuration.
//
// Configure the MMU to use 4 KiB granules for both the kernel and user address
// spaces.
//
// With a 4 KiB granule size, bits 47:39 of the address are the Level 1
// translation index. So, just configure T0SZ and T1SZ to mask off the top 16
// bits of the address.
//
// The kernel address space will span the 256 TiB from 0xffff_0000_0000_000 to
// 0xffff_ffff_ffff_ffff while the user address space will span the 256 TiB
// from 0x0000_0000_0000_0000 to 0x0000_ffff_ffff_ffff.
#define TCR_EL1_T0SZ   16
#define TCR_EL1_T1SZ   (TCR_EL1_T0SZ << 16)
#define TCR_EL1_TG0_4K (0 << 14)
#define TCR_EL1_TG1_4K (2 << 30)
#define TCR_EL1_VALUE  (TCR_EL1_T0SZ | TCR_EL1_T1SZ | TCR_EL1_TG0_4K | TCR_EL1_TG1_4K)

// EL1 memory attribute indirection register configuration.
//
//   * Configure attribute 0 to tag pages as non Gathering, non Re-ordering,
//     non Early Write Acknowledgement. This is a restriction we will apply to
//     the peripheral memory to ensure writes are done exactly as specified
//     with no relative re-ordering and we get an acknowledgement from the
//     peripheral.
//
//   * For now, normal memory will be non-caching.
#define MT_DEVICE_nGnRnE       0x0
#define MT_NORMAL_NC           0x1
#define MT_DEVICE_nGnRnE_FLAGS 0x00
#define MT_NORMAL_NC_FLAGS     0x44
#define MAIR_EL1_VALUE                                                                             \
  ((MT_DEVICE_nGnRnE_FLAGS << (8 * MT_DEVICE_nGnRnE)) | (MT_NORMAL_NC_FLAGS << (8 * MT_NORMAL_NC)))

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

#endif
