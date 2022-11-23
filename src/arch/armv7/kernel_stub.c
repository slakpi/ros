#include <stdint.h>
#include "ros_kernel.h"

#if defined __aarch64__
#error "Attempting to use AArch32 kernel stub for AArch64 architecture."
#endif

#define PERIPHERAL_BASE 0x3F000000

#if RPI_VERSION > 2
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
