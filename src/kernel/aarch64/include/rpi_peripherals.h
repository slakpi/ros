#if !defined RPI_PERIPHERALS_H
#define RPI_PERIPHERALS_H

#include "rpi_common.h"

// AArch64 is only available on Raspberry Pi models 3 and higher.
#if !defined RPI_VERSION
#error "Raspberry Pi board version not defined."
#elif RPI_VERSION == 3
#define PERIPHERAL_BASE RPI_PERIPHERAL_BASE
#elif RPI_VERSION > 3
#define PERIPHERAL_BASE RPI4_PERIPHERAL_BASE
#else
#error "Invalid Raspberry Pi board version for AArch64."
#endif

#endif
