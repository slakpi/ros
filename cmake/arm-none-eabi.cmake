set(CMAKE_SYSTEM_NAME Generic)
set(CMAKE_SYSTEM_PROCESSOR armv7)

set(cross_compiler ${TC_PATH}/bin/arm-none-eabi-)

set(CMAKE_C_COMPILER ${cross_compiler}gcc)
set(CMAKE_CXX_COMPILER ${cross_compiler}g++)
set(CMAKE_ASM_COMPILER ${cross_compiler}gcc)

set(CMAKE_TRY_COMPILE_TARGET_TYPE STATIC_LIBRARY)

set(CMAKE_OBJCOPY ${TC_PATH}${cross_compiler}objcopy
    CACHE FILEPATH "The toolchain objcopy command " FORCE)

set(CMAKE_OBJDUMP ${TC_PATH}${CROSS_COMPILE}objdump
    CACHE FILEPATH "The toolchain objdump command " FORCE )

set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -nostartfiles")
set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -nostdlib")
set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -mfloat-abi=hard")
set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -mfpu=neon")
set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -mcpu=cortex-a7")

set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS}" CACHE STRING "")
set(CMAKE_ASM_FLAGS "${CMAKE_C_FLAGS}" CACHE STRING "")
