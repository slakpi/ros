set(CMAKE_SYSTEM_NAME Generic)
set(CMAKE_SYSTEM_PROCESSOR armv7)

set(cross_compiler ${TC_PATH}/gnu-arm-none-eabi/bin/arm-none-eabi-)

set(CMAKE_ASM_COMPILER ${cross_compiler}gcc)
set(CMAKE_C_COMPILER ${cross_compiler}gcc)
set(CMAKE_OBJCOPY ${cross_compiler}objcopy)
set(CMAKE_OBJDUMP ${cross_compiler}objdump)
set(CMAKE_TRY_COMPILE_TARGET_TYPE STATIC_LIBRARY)
