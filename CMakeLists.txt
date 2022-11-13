cmake_minimum_required(VERSION 3.16)

project(ros C CXX ASM)
set(CMAKE_C_STANDARD 11)
set(CMAKE_CXX_STANDARD 20)

add_subdirectory(src/arch)
add_subdirectory(src/ros)
