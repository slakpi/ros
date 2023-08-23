//! ARMv7a MMU Configuration

#if !defined MMU_H
#define MMU_H

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

#endif
