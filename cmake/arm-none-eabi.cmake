set(CMAKE_SYSTEM_NAME Generic)
set(CMAKE_SYSTEM_PROCESSOR armv7) # 32-bit Armv7

set(cross_compiler ${TC_PATH}/bin/arm-none-eabi-)

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
if(${RPI_VERSION} STREQUAL 3)
  set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -mcpu=cortex-a53")
else()
  set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -mcpu=cortex-a7")
endif()

set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS}" CACHE STRING "")
set(CMAKE_ASM_FLAGS "${CMAKE_C_FLAGS}" CACHE STRING "")

# Set the Rust target
set(Rust_CARGO_TARGET armv7a-none-eabi)

# Set the kernel image file name; kernel7.img is used for 32-bit Rpi 2 & 3
set(ROS_KERNEL_IMAGE_FILE kernel7.img)
