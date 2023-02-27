set(CMAKE_SYSTEM_NAME Generic)
set(CMAKE_SYSTEM_PROCESSOR aarch64) # 64-bit AArch64

set(cross_compiler ${TC_PATH}/bin/aarch64-none-elf-)

set(CMAKE_ASM_COMPILER ${cross_compiler}gcc)
set(CMAKE_C_COMPILER ${cross_compiler}gcc)
set(CMAKE_TRY_COMPILE_TARGET_TYPE STATIC_LIBRARY)

set(CMAKE_OBJCOPY ${cross_compiler}objcopy
    CACHE FILEPATH "The toolchain objcopy command " FORCE)
set(CMAKE_OBJDUMP ${cross_compiler}objdump
    CACHE FILEPATH "The toolchain objdump command " FORCE )

set(CMAKE_ASM_FLAGS "${CMAKE_ASM_FLAGS} -nostdlib -nostartfiles")

# Turn off hardware floating-point and SIMD. The kernel does not allow floating-
# point or vector instructions.
set(CMAKE_ASM_FLAGS "${CMAKE_ASM_FLAGS} -mcpu=+nofp+nosimd")

# Set the CPU models for the Raspberry Pi model. AArch64 is only supported the
# Raspberry Pi 3 and higher. NOTE: The Zero 2W model uses the same processor as
# the 3, the SoC just has less RAM.
if("${RPI_VERSION}" STREQUAL "4")
  set(CMAKE_ASM_FLAGS "${CMAKE_ASM_FLAGS} -mcpu=cortex-a72")
else()
  set(CMAKE_ASM_FLAGS "${CMAKE_ASM_FLAGS} -mcpu=cortex-a53")
endif()

set(CMAKE_C_FLAGS "${CMAKE_ASM_FLAGS}")
set(CMAKE_C_FLAGS "${CMAKE_ASM_FLAGS}" CACHE STRING "")
set(CMAKE_ASM_FLAGS "${CMAKE_ASM_FLAGS}" CACHE STRING "")

# Set the Rust target. Use the software floating-point variant. The kernel does
# not allow floating-point or vector instructions.
set(Rust_CARGO_TARGET aarch64-unknown-none-softfloat)

# The Raspberry Pi bootloader expects 64-bit kernels to be named `kernel8.img`
# and places them at 0x80000 by default.
set(ROS_KERNEL_IMAGE_FILE kernel8.img)
set(ROS_KERNEL_BASE_ADDRESS 0x80000)

# The canonical 64-bit kernel segment is the top 128 TiB
set(ROS_VIRTUAL_BASE_ADDRESS 0xffff800000000000)

# QEMU_BUILD is not used by AArch64. It has no effect, so go ahead and just
# quiet the warning about it being unused.
set(ignore_QEMU_BUILD ${QEMU_BUILD})
