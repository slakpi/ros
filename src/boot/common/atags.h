#pragma once

#include "ros_kernel.h"
#include <stdbool.h>
#include <stdint.h>

/**
 * @fn read_atags(ROSKernelInit *init, uintptr_t start)
 * @brief   Configures the ROS kernel initialization struct with ATAGs.
 * @param[out] init The ROS kernel intialization struct.
 * @param[in] start The start address of the ATAGs.
 * @returns True if @a start points to a valid ATAG list, false otherwise.
 */
bool read_atags(ROSKernelInit *init, uintptr_t start);
