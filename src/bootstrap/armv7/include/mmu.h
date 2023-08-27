//! ARMv7a MMU Configuration

#if !defined MMU_H
#define MMU_H

// TTBCR value. See B4.1.153. A value of 2 disables extended addresses, enables
// TTBR1 translation, and enables TTBR0 translation. If the top 2 bits of the
// virtual address are 0, TTBR0 is used. Otherwise, TTBR1 is used.
#define TTBCR_VALUE 0x2

// Page descriptor flags. See B3.5.1.
#define MM_TYPE_PAGE_TABLE 0x1
#define MM_TYPE_PAGE       0x2
#define MM_TYPE_BLOCK      0x2
#define MM_ACCESS_FLAG     (1 << 15)
#define MM_ACCESS_RW       (0b00 << 10)
#define MM_ACCESS_RO       (0b10 << 10)
#define MM_DEVICE_CB       (0b01 << 2)
#define MM_NORMAL_CB       (0b10 << 2)

#define MMU_NORMAL_RO_FLAGS (MM_TYPE_BLOCK | MM_ACCESS_RO | MM_NORMAL_CB | MM_ACCESS_FLAG)
#define MMU_NORMAL_RW_FLAGS (MM_TYPE_BLOCK | MM_ACCESS_RW | MM_NORMAL_CB | MM_ACCESS_FLAG)
#define MMU_DEVICE_RO_FLAGS (MM_TYPE_BLOCK | MM_ACCESS_RO | MM_DEVICE_CB | MM_ACCESS_FLAG)
#define MMU_DEVICE_RW_FLAGS (MM_TYPE_BLOCK | MM_ACCESS_RW | MM_DEVICE_CB | MM_ACCESS_FLAG)

// SCTLR bit to enable the MMU. See B4.1.130.
#define MMU_ENABLE 0x1

// DACR setup. See B4.1.43. Only using domain 0 in client mode (access
// permissions are checked).
#define DACR_VALUE 0x3

#endif
