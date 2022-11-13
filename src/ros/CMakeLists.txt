add_library(ros STATIC ros_kernel.c)
target_include_directories(ros PUBLIC ${CMAKE_CURRENT_LIST_DIR})

add_library(ROS::Lib ALIAS ros)
