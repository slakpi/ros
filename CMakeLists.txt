cmake_minimum_required(VERSION 3.19)

project(ros C ASM)
set(CMAKE_C_STANDARD 11)

if(CMAKE_SYSTEM_PROCESSOR STREQUAL "aarch64")
elseif(CMAKE_SYSTEM_PROCESSOR STREQUAL "armv7")
endif()

add_subdirectory(cmake/corrosion)

add_subdirectory(src/arch)
add_subdirectory(src/kernel)
