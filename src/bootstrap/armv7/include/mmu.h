//! ARMv7a MMU Configuration

#if !defined MMU_H
#define MMU_H

// TTBCR value. See B4.1.153 and B3.6.4. The EAE bit enables the long descriptor
// extension. A value of 2 for TTBCR.T1SZ sets up the MMU to use TTBR1 when the
// top two bits of a virtual address are BOTH 1. TTBR0 is used otherwise. A
// value of 0 for TTBCR.A1 tells the MMU that TTBR0 defines address space IDs.
#define TTBCR_EAE   (0b1 << 31)
#define TTBCR_A1    (0x0 << 22)
#define TTBCR_T1SZ  (0b10 << 16)
#define TTBCR_T0SZ  (0x0 << 0)
#define TTBCR_VALUE (TTBCR_EAE | TTBCR_A1 | TTBCR_T1SZ | TTBCR_T0SZ)

// Page descriptor flags. See B3.5.1. The Access Flag Enable bit should be set
// to 1 in SCTLR to use the 2-bit access control model where AP[2] (bit 15)
// disables writing and A[1] (bit 11) enables unpriviledged access. AP[0] (bit
// 10) is the Access Flag.
#define MM_TYPE_PAGE_TABLE 0x1
#define MM_TYPE_PAGE       0x2
#define MM_TYPE_BLOCK      0x2
#define MM_ACCESS_FLAG     (0b1 << 10)
#define MM_ACCESS_RW       (0b0 << 15)
#define MM_ACCESS_RO       (0b1 << 15)
#define MM_DEVICE_CB       (0b01 << 2)
#define MM_NORMAL_CB       (0b10 << 2)

#define MMU_NORMAL_RO_FLAGS (MM_TYPE_BLOCK | MM_ACCESS_RO | MM_NORMAL_CB | MM_ACCESS_FLAG)
#define MMU_NORMAL_RW_FLAGS (MM_TYPE_BLOCK | MM_ACCESS_RW | MM_NORMAL_CB | MM_ACCESS_FLAG)
#define MMU_DEVICE_RO_FLAGS (MM_TYPE_BLOCK | MM_ACCESS_RO | MM_DEVICE_CB | MM_ACCESS_FLAG)
#define MMU_DEVICE_RW_FLAGS (MM_TYPE_BLOCK | MM_ACCESS_RW | MM_DEVICE_CB | MM_ACCESS_FLAG)

#endif
