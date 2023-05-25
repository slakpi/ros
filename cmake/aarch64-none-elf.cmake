set(CMAKE_SYSTEM_NAME Generic)
set(CMAKE_SYSTEM_PROCESSOR aarch64)

if(DEFINED RPI_VERSION AND NOT ("${RPI_VERSION}" MATCHES "^(3|4)$"))
  message(FATAL_ERROR "Unsupported Raspberry Pi board version.")
endif()

set(cross_compiler ${TC_PATH}/gnu-aarch64-none-elf/bin/aarch64-none-elf-)

set(CMAKE_ASM_COMPILER ${cross_compiler}gcc)
set(CMAKE_C_COMPILER ${cross_compiler}gcc)
set(CMAKE_TRY_COMPILE_TARGET_TYPE STATIC_LIBRARY)

set(CMAKE_OBJCOPY ${cross_compiler}objcopy
    CACHE FILEPATH "The toolchain objcopy command " FORCE)
set(CMAKE_OBJDUMP ${cross_compiler}objdump
    CACHE FILEPATH "The toolchain objdump command " FORCE )

set(CMAKE_ASM_FLAGS "${CMAKE_ASM_FLAGS} -nostdlib -nostartfiles")

set(Rust_RUSTFLAGS "")

# If a Raspberry Pi version is specified, set the CPU model. NOTE: The Zero 2W
# model uses the same processor as the 3, the SoC just has less RAM.
if("${RPI_VERSION}" STREQUAL "4")
  set(CMAKE_ASM_FLAGS "${CMAKE_ASM_FLAGS} -mcpu=cortex-a72")
  set(Rust_RUSTFLAGS ${Rust_RUSTFLAGS} -Ctarget-cpu=cortex-a72)
elseif("${RPI_VERSION}" STREQUAL "3")
  set(CMAKE_ASM_FLAGS "${CMAKE_ASM_FLAGS} -mcpu=cortex-a53")
  set(Rust_RUSTFLAGS ${Rust_RUSTFLAGS} -Ctarget-cpu=cortex-a53)
endif()

# Hardware floating-point and SIMD are enabled by default with AArch64. If the
# kernel uses those registers, however, they will need to be saved when software
# exceptions trap into the kernel. This wastes time and memory, so just disable
# both for the kernel. NOTE: The Rust toolchain only supports software floating-
# point, so only SIMD needs to be disabled.
set(CMAKE_ASM_FLAGS "${CMAKE_ASM_FLAGS} -march=armv8-a+nofp+nosimd")
set(Rust_RUSTFLAGS ${Rust_RUSTFLAGS} -Ctarget-feature=-neon)

set(CMAKE_C_FLAGS "${CMAKE_ASM_FLAGS}")
set(CMAKE_C_FLAGS "${CMAKE_ASM_FLAGS}" CACHE STRING "")
set(CMAKE_ASM_FLAGS "${CMAKE_ASM_FLAGS}" CACHE STRING "")

# Set the Rust target. Use the software floating-point variant.
set(Rust_CARGO_TARGET aarch64-unknown-none-softfloat)

# If a Raspberry Pi version is specified, set the kernel image name for the
# bootloader.
if(DEFINED RPI_VERSION)
  set(ROS_KERNEL_IMAGE_FILE kernel8.img)
else()
  set(ROS_KERNEL_IMAGE_FILE kernel.img)
endif()

# If a Raspberry Pi version is specified, set the kernel base address for the
# bootloader. The bootloader expects 64-bit images at 0x80000 by default.
if(DEFINED RPI_VERSION)
  set(ROS_KERNEL_BASE_ADDRESS 0x80000)
elseif(NOT DEFINED ROS_KERNEL_BASE_ADDRESS)
  set(ROS_KERNEL_BASE_ADDRESS 0x0)
endif()

# The canonical 64-bit kernel segment is the top 256 TiB
set(ROS_VIRTUAL_BASE_ADDRESS 0xffff000000000000)

# QEMU_BUILD is not used by AArch64. It has no effect, so go ahead and just
# quiet the warning about it being unused.
set(ignore_QEMU_BUILD ${QEMU_BUILD})
