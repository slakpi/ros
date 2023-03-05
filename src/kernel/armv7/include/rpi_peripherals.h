#if !defined RPI_PERIPHERALS_H
#define RPI_PERIPHERALS_H

#include "rpi_common.h"

#if !defined RPI_VERSION
#error "Raspberry Pi board version not defined."
#elif RPI_VERSION < 2
#error "Invalid Raspberry Pi board version for ARMv7."
#elif RPI_VERSION < 4
#define PERIPHERAL_BASE RPI_PERIPHERAL_BASE
#else
#define PERIPHERAL_BASE RPI4_PERIPHERAL_BASE
#endif

#endif
