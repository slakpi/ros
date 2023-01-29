cmake_minimum_required(VERSION 3.19)

project(ros ASM)

add_subdirectory(cmake/corrosion)
add_subdirectory(src/kernel)
add_subdirectory(src/ros_kernel)
add_subdirectory(src/ros_user)
