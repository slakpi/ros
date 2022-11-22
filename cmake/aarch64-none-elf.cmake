set(CMAKE_SYSTEM_NAME Generic)
set(CMAKE_SYSTEM_PROCESSOR aarch64) # 64-bit AArch64

set(cross_compiler ${TC_PATH}/bin/aarch64-none-elf-)

set(CMAKE_C_COMPILER ${cross_compiler}gcc)
set(CMAKE_CXX_COMPILER ${cross_compiler}g++)
set(CMAKE_ASM_COMPILER ${cross_compiler}gcc)

set(CMAKE_TRY_COMPILE_TARGET_TYPE STATIC_LIBRARY)

set(CMAKE_OBJCOPY ${cross_compiler}objcopy
    CACHE FILEPATH "The toolchain objcopy command " FORCE)

set(CMAKE_OBJDUMP ${cross_compiler}objdump
    CACHE FILEPATH "The toolchain objdump command " FORCE )

set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -nostartfiles")
set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -nostdlib")
# -mfloat-abi is invalid for AArch64; hardware floating-point is the default.
# -mfpu is ignored for AArch64; NEON is the default.
if("${RPI_VERSION}" STREQUAL "4")
  set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -mcpu=cortex-a72")
elseif("${RPI_VERSION}" STREQUAL "3")
  set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -mcpu=cortex-a53")
endif()

set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS}" CACHE STRING "")
set(CMAKE_ASM_FLAGS "${CMAKE_C_FLAGS}" CACHE STRING "")

# Set the Rust target
set(Rust_CARGO_TARGET aarch64-unknown-none)

# Set the kernel image file name; kernel8.img is used for 64-bit Rpi 3 & 4
set(ROS_KERNEL_IMAGE_FILE kernel8.img)
