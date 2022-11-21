#include <stdint.h>
#include "ros_kernel.h"

#if __ARM_ARCH != 8
#error "Attempting to use Armv8 kernel stub for non-Armv8 architecture."
#endif

void kernel_stub(uint32_t r0, uint32_t r1, uint32_t atags) {
  ros_main();
}
