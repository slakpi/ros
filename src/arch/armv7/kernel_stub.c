#include <stdint.h>
#include "ros_kernel.h"

#if __ARM_ARCH != 7
#error "Attempting to use Armv7 kernel stub for non-Armv7 architecture."
#endif

void kernel_stub(uint32_t r0, uint32_t r1, uint32_t atags) {
  ros_main();
}
