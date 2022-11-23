#include <stdint.h>
#include "ros_kernel.h"

#if !defined __aarch64__
#error "Attempting to use AArch64 kernel stub for non-AArch64 architecture."
#endif

#define RPI4_PERIPHERAL_BASE 0xFE000000
#define RPI3_PERIPHERAL_BASE 0x3F000000

#if (!defined RPI_VERSION) || (RPI_VERSION == 3)
#define PERIPHERAL_BASE RPI3_PERIPHERAL_BASE
#elif RPI_VERSION == 4
#define PERIPHERAL_BASE RPI4_PERIPHERAL_BASE
#else
#error "Invalid Raspberry Pi board version for AArch64."
#endif

/**
 * @fn void kernel_stub(uint64_t dtb_ptr32)
 * @brief   AArch64 kernel stub.
 * @details Should eventually do architecture-specific stuff with the device
 *          tree and pass it on to Rustland.
 * @param[in] dtb_ptr32 32-bit pointer to the device tree blob
 */
void kernel_stub(uint64_t dtb_ptr32) {
  const ROSKernelInit init = {
    .peripheral_base = PERIPHERAL_BASE
  };

  ros_kernel(&init);
}
