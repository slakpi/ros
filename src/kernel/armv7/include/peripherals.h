#if !defined PERIPHERALS_H
#define PERIPHERALS_H

#if RPI_VERSION > 2
#error "Invalid Raspberry Pi board version for ARMv7."
#endif

#define PERIPHERAL_BASE 0x3f000000

#endif
