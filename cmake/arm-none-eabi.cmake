set(CMAKE_SYSTEM_NAME Generic)
set(CMAKE_SYSTEM_PROCESSOR armv7) # 32-bit ARMv7

if(NOT DEFINED RPI_VERSION)
  set(RPI_VERSION "2")
endif()

set(cross_compiler ${TC_PATH}/bin/arm-none-eabi-)

set(CMAKE_ASM_COMPILER ${cross_compiler}gcc)
set(CMAKE_C_COMPILER ${cross_compiler}gcc)
set(CMAKE_TRY_COMPILE_TARGET_TYPE STATIC_LIBRARY)

set(CMAKE_OBJCOPY ${cross_compiler}objcopy
    CACHE FILEPATH "The toolchain objcopy command " FORCE)
set(CMAKE_OBJDUMP ${cross_compiler}objdump
    CACHE FILEPATH "The toolchain objdump command " FORCE )

set(CMAKE_ASM_FLAGS "${CMAKE_ASM_FLAGS} -nostdlib -nostartfiles")

# It is not necessary to turn off hardware floating-point. This toolchain only
# supports software floating-point.

# Set the CPU models for the Raspberry Pi model. NOTE: The Zero 2W model uses
# the same processor as the 3, the SoC just has less RAM.
if("${RPI_VERSION}" STREQUAL "4")
  set(CMAKE_ASM_FLAGS "${CMAKE_ASM_FLAGS} -mcpu=cortex-a72")
elseif("${RPI_VERSION}" STREQUAL "3")
  set(CMAKE_ASM_FLAGS "${CMAKE_ASM_FLAGS} -mcpu=cortex-a53")
elseif("${RPI_VERSION}" STREQUAL "2")
  set(CMAKE_ASM_FLAGS "${CMAKE_ASM_FLAGS} -mcpu=cortex-a7")
endif()

set(CMAKE_C_FLAGS "${CMAKE_ASM_FLAGS}")
set(CMAKE_C_FLAGS "${CMAKE_ASM_FLAGS}" CACHE STRING "")
set(CMAKE_ASM_FLAGS "${CMAKE_ASM_FLAGS}" CACHE STRING "")

# Set the Rust target. Use the software floating-point variant. The kernel does
# not allow floating-point or vector instructions.
set(Rust_CARGO_TARGET armv7a-none-eabi)

# The Raspberry Pi bootloader expects 32-bit kernels to be named:
#
#   Raspberry Pi      Kernel Name
#   -------------------------------
#   2 or 3            kernel7.img
#   4                 kernel7l.img
#
# NOTE: Models 0 and 1 are not supported by ROS.
if("${RPI_VERSION}" STREQUAL "2" OR "${RPI_VERSION}" STREQUAL "3")
  set(ROS_KERNEL_IMAGE_FILE kernel7.img)
else()
  set(ROS_KERNEL_IMAGE_FILE kernel7l.img)
endif()

# The Raspberry Pi bootloader places 32-bit kernel images at 0x8000 by default.
# QEMU, however, places them at 0x10000.
if(${QEMU_BUILD})
  set(ROS_KERNEL_BASE_ADDRESS 0x10000)
else()
  set(ROS_KERNEL_BASE_ADDRESS 0x8000)
endif()

# The canonical 3:1 split kernel segment is the top 1 GiB.
set(ROS_VIRTUAL_BASE_ADDRESS 0xc0000000)
