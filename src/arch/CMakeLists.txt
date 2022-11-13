if(CMAKE_SYSTEM_PROCESSOR STREQUAL "aarch64")
  add_executable(kernel aarch64/boot.S aarch64/kernel_stub.c)
  target_link_options(kernel PUBLIC -T ${CMAKE_CURRENT_LIST_DIR}/aarch64/boot.ld)
elseif(CMAKE_SYSTEM_PROCESSOR STREQUAL "armv7")
add_executable(kernel armv7/boot.S armv7/kernel_stub.c)
  target_link_options(kernel PUBLIC -T ${CMAKE_CURRENT_LIST_DIR}/armv7/boot.ld)
endif()

target_link_libraries(kernel PRIVATE ROS::Lib)
