cmake_minimum_required(VERSION 3.19)

list(APPEND CMAKE_MODULE_PATH ${CMAKE_CURRENT_SOURCE_DIR}/cmake)
include(platform)

project(ros ASM)

validate_platform()

rust_arch_target(Rust_CARGO_TARGET)
add_subdirectory(cmake/corrosion)

add_subdirectory(src/ros_kernel)
add_subdirectory(src/ros_user)
add_subdirectory(src/start)
