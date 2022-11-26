#pragma once

#include <stdint.h>

/**
 * 
 */
typedef struct {
  uintptr_t base;
  uintptr_t size;
} ROSMemoryRegion;

/**
 * @struct  ROSKernelInit
 * @brief   Architecture-specific initialization values.
 * @details See definition in `ros_kernel.rs`.
 */
typedef struct {
  uintptr_t peripheral_base;
  ROSMemoryRegion mem_regions[16];
} ROSKernelInit;

/**
 * @fn ros_kernel(const ROSKernelInit *init)
 * @brief Rust ROS kernel entry point.
 */
void ros_kernel(const ROSKernelInit *init);
