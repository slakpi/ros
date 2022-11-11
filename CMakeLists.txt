cmake_minimum_required(VERSION 3.16)

project(ros C CXX ASM)
set(CMAKE_C_STANDARD 11)
set(CMAKE_CXX_STANDARD 20)

add_subdirectory(src/kernel_stub)
add_subdirectory(src/kernel)
