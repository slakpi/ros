#-------------------------------------------------------------------------------
# Build the kernel executable.
#-------------------------------------------------------------------------------
add_executable(kernel ${CMAKE_SYSTEM_PROCESSOR}/boot.S ${CMAKE_SYSTEM_PROCESSOR}/kernel_stub.c)
target_link_options(kernel PUBLIC -T ${CMAKE_CURRENT_LIST_DIR}/${CMAKE_SYSTEM_PROCESSOR}/boot.ld)
target_link_libraries(kernel PRIVATE ROS::Lib)

#-------------------------------------------------------------------------------
# Build the raw binary image file.
#-------------------------------------------------------------------------------
add_custom_target(
  kernel_img
  ${CMAKE_OBJCOPY} $<TARGET_FILE:kernel> -O binary ${CMAKE_CURRENT_BINARY_DIR}/${ROS_KERNEL_IMAGE_FILE}
  BYPRODUCTS ${CMAKE_CURRENT_BINARY_DIR}/${ROS_KERNEL_IMAGE_FILE}
  DEPENDS kernel
  COMMENT "Making kernel image...")
