#-------------------------------------------------------------------------------
# Build the kernel executable.
#-------------------------------------------------------------------------------
add_executable(
  kernel ${CMAKE_SYSTEM_PROCESSOR}/boot.S
         ${CMAKE_SYSTEM_PROCESSOR}/exceptions.S
         ${CMAKE_SYSTEM_PROCESSOR}/pages.S)
target_compile_definitions(kernel PRIVATE RPI_VERSION=${RPI_VERSION})
target_link_libraries(kernel PRIVATE ROS::Kernel)
target_link_options(
  kernel PUBLIC -T ${CMAKE_CURRENT_LIST_DIR}/boot.ld -Ttext ${ROS_KERNEL_BASE_ADDRESS})

#-------------------------------------------------------------------------------
# Build the raw binary image file.
#-------------------------------------------------------------------------------
set(kernel_image ${CMAKE_CURRENT_BINARY_DIR}/${ROS_KERNEL_IMAGE_FILE})
add_custom_target(
  kernel_img
  ALL
  ${CMAKE_OBJCOPY} $<TARGET_FILE:kernel> -O binary ${kernel_image}
  BYPRODUCTS ${kernel_image}
  DEPENDS kernel
  COMMENT "Making kernel image...")
