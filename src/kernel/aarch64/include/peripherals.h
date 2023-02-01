#if !defined PERIPHERALS_H
#define PERIPHERALS_H

#define RPI4_PERIPHERAL_BASE  0xfe000000
#define RPI3_PERIPHERAL_BASE  0x3f000000
#define PERIPHERAL_BLOCK_SIZE 0x1000000

#if (!defined RPI_VERSION) || (RPI_VERSION == 3)
#define PERIPHERAL_BASE RPI3_PERIPHERAL_BASE
#elif RPI_VERSION > 3
#define PERIPHERAL_BASE RPI4_PERIPHERAL_BASE
#else
#error "Invalid Raspberry Pi board version for AArch64."
#endif

#endif
