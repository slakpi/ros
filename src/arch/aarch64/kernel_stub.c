#include <stdint.h>
#include "ros_kernel.h"

#if !defined __aarch64__
#error "Attempting to use AArch64 kernel stub for non-AArch64 architecture."
#endif

void kernel_stub(uint64_t dtb_ptr32, uint64_t x1, uint64_t x2, uint64_t x3) {
  ros_main();
}
