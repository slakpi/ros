set(CMAKE_SYSTEM_NAME Generic)
set(CMAKE_SYSTEM_PROCESSOR aarch64) # 64-bit AArch64

set(cross_compiler ${TC_PATH}/bin/aarch64-none-elf-)

set(CMAKE_ASM_COMPILER ${cross_compiler}gcc)

set(CMAKE_TRY_COMPILE_TARGET_TYPE STATIC_LIBRARY)

set(CMAKE_OBJCOPY ${cross_compiler}objcopy
    CACHE FILEPATH "The toolchain objcopy command " FORCE)

set(CMAKE_OBJDUMP ${cross_compiler}objdump
    CACHE FILEPATH "The toolchain objdump command " FORCE )

set(CMAKE_ASM_FLAGS "${CMAKE_ASM_FLAGS} -nostdlib -nostartfiles")
# -mfloat-abi is invalid for AArch64; hardware floating-point is the default.
# -mfpu is ignored for AArch64; NEON is the default.
# Turn off hardware floating-point and SIMD. The kernel does not allow floating-
# point or vector instructions.
set(CMAKE_ASM_FLAGS "${CMAKE_ASM_FLAGS} -mcpu=+nofp+nosimd")
if("${RPI_VERSION}" STREQUAL "4")
  set(CMAKE_ASM_FLAGS "${CMAKE_ASM_FLAGS} -mcpu=cortex-a72")
elseif("${RPI_VERSION}" STREQUAL "3")
  set(CMAKE_ASM_FLAGS "${CMAKE_ASM_FLAGS} -mcpu=cortex-a53")
endif()

set(CMAKE_C_FLAGS "${CMAKE_ASM_FLAGS}")
set(CMAKE_C_FLAGS "${CMAKE_ASM_FLAGS}" CACHE STRING "")
set(CMAKE_ASM_FLAGS "${CMAKE_ASM_FLAGS}" CACHE STRING "")

# Set the Rust target. Use the software floating-point variant. The kernel does
# not allow floating-point or vector instructions.
set(Rust_CARGO_TARGET aarch64-unknown-none-softfloat)

set(ROS_KERNEL_IMAGE_FILE kernel8.img)

# QEMU_BUILD is not used by AArch64. It has no effect, so go ahead and just
# quiet the warning about it being unused.
set(ignore_QEMU_BUILD ${QEMU_BUILD})
