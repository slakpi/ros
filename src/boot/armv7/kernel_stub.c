#include "atags.h"
#include "ros_kernel.h"
#include <stdbool.h>
#include <stdint.h>

#if defined __aarch64__
#error "Attempting to use AArch32 kernel stub for AArch64 architecture."
#endif

#if RPI_VERSION > 2
#error "Invalid Raspberry Pi board version."
#endif

#define PERIPHERAL_BASE 0x3F000000

/**
 * @fn void kernel_stub(uint32_t r0, uint32_t r1, uint32_t atags)
 * @brief Armv7 kernel stub.
 * @param[in] r0    Zero
 * @param[in] r1    Machine ID
 * @param[in] atags ATAG list
 */
void kernel_stub(uint32_t r0, uint32_t r1, uint32_t atags) {
  ROSKernelInit init;
  
  init.peripheral_base = PERIPHERAL_BASE;
  
  if (!read_atags(&init, atags)) {
    return; // TODO: Check for device tree.
  }
  
  ros_kernel(&init);
}
