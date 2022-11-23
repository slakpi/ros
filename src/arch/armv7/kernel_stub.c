#include <stdint.h>
#include "ros_kernel.h"

#if __ARM_ARCH != 7
#error "Attempting to use Armv7 kernel stub for non-Armv7 architecture."
#endif

#define PERIPHERAL_BASE 0x3F000000

#if RPI_VERSION > 3
#error "Invalid Raspberry Pi board version."
#endif

/**
 * @fn void kernel_stub(uint32_t r0, uint32_t r1, uint32_t atags)
 * @brief   Armv7 kernel stub.
 * @details Should eventually do architecture-specific stuff with the ATAGS
 *          and pass it on to Rustland.
 * @param[in] r0    Zero
 * @param[in] r1    Machine ID
 * @param[in] atags ATAGS
 */
void kernel_stub(uint32_t r0, uint32_t r1, uint32_t atags) {
  const ROSKernelInit init = {
    .peripheral_base = PERIPHERAL_BASE
  };

  ros_kernel(&init);
}
