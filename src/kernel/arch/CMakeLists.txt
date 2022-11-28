add_library(ros_arch STATIC ${CMAKE_SYSTEM_PROCESSOR}/foo.S)
add_library(ROS::Arch ALIAS ros_arch)
