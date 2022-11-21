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
set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -mcpu=cortex-a72") # Rpi 4B

set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS}" CACHE STRING "")
set(CMAKE_ASM_FLAGS "${CMAKE_C_FLAGS}" CACHE STRING "")
