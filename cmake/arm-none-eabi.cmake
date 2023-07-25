# Set the assembly target. Unlike the AArch64 toolchain, the ARM toolchain only
# supports software floating-point and does not enable SIMD by default.
set(CMAKE_SYSTEM_NAME Generic)
set(CMAKE_SYSTEM_PROCESSOR armv7)

# Set the Rust target. Use the software floating-point variant.
set(Rust_CARGO_TARGET armv7a-none-eabi)

if(DEFINED RPI_VERSION AND NOT ("${RPI_VERSION}" MATCHES "^(2|3|4)$"))
  message(FATAL_ERROR "Unsupported Raspberry Pi board version.")
endif()

set(cross_compiler ${TC_PATH}/gnu-arm-none-eabi/bin/arm-none-eabi-)

set(CMAKE_ASM_COMPILER ${cross_compiler}gcc)
set(CMAKE_C_COMPILER ${cross_compiler}gcc)
set(CMAKE_OBJCOPY ${cross_compiler}objcopy)
set(CMAKE_OBJDUMP ${cross_compiler}objdump)
set(CMAKE_TRY_COMPILE_TARGET_TYPE STATIC_LIBRARY)

# -nostdlib: Do not link the standard library.
# -nostartfiles: Do not use the standard library startup files.
# -z noexecstack: Prevents executing code in a stack.
add_link_options(-nostdlib -nostartfiles -z noexecstack)

# If a Raspberry Pi version is specified, set the CPU model. NOTE: The Zero 2W
# model uses the same processor as the 3, the SoC just has less RAM.
if("${RPI_VERSION}" STREQUAL "4")
  add_compile_options(-mcpu=cortex-a72)
  set(Rust_RUSTFLAGS ${Rust_RUSTFLAGS} -Ctarget-cpu=cortex-a72)
elseif("${RPI_VERSION}" STREQUAL "3")
  add_compile_options(-mcpu=cortex-a53)
  set(Rust_RUSTFLAGS ${Rust_RUSTFLAGS} -Ctarget-cpu=cortex-a53)
elseif("${RPI_VERSION}" STREQUAL "2")
  add_compile_options(-mcpu=cortex-a7)
  set(Rust_RUSTFLAGS ${Rust_RUSTFLAGS} -Ctarget-cpu=cortex-a7)
endif()

# If a Raspberry Pi version is specified, set the kernel image name for the
# bootloader:
#
#   Raspberry Pi      Kernel Name
#   -------------------------------
#   2 or 3            kernel7.img
#   4                 kernel7l.img
#
# NOTE: Models 0 and 1 are not supported by ROS.
if("${RPI_VERSION}" MATCHES "^(2|3)$")
  set(ROS_KERNEL_IMAGE_FILE kernel7.img)
elseif("${RPI_VERSION}" STREQUAL "4")
  set(ROS_KERNEL_IMAGE_FILE kernel7l.img)
else()
  set(ROS_KERNEL_IMAGE_FILE kernel.img)
endif()

# If a Raspberry Pi version is specified, set the kernel base address for the
# bootloader. The bootloader expects 32-bit images at 0x8000 by default. QEMU,
# however, expects them at 0x10000. If a Raspberry Pi version is not specified,
# just default to 0x0 if not specified.
if(DEFINED RPI_VERSION)
  if(${QEMU_BUILD})
    set(ROS_KERNEL_BASE_ADDRESS 0x10000)
  else()
    set(ROS_KERNEL_BASE_ADDRESS 0x8000)
  endif()
elseif(NOT DEFINED ROS_KERNEL_BASE_ADDRESS)
  set(ROS_KERNEL_BASE_ADDRESS 0x0)
endif()

# The canonical 3:1 split kernel segment is the top 1 GiB.
set(ROS_VIRTUAL_BASE_ADDRESS 0xc0000000)
