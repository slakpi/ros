#-------------------------------------------------------------------------------
# Build the kernel executable.
#-------------------------------------------------------------------------------
add_executable(kernel ${CMAKE_SYSTEM_PROCESSOR}/boot.S ${CMAKE_SYSTEM_PROCESSOR}/kernel_stub.c)
target_link_options(kernel PUBLIC -T ${CMAKE_CURRENT_LIST_DIR}/${CMAKE_SYSTEM_PROCESSOR}/boot.ld)
target_link_libraries(kernel PRIVATE ROS::Lib)

#-------------------------------------------------------------------------------
# Build the kernel image:
#
#   . kernel.img      32-bit Rpi 1 & Rpi Zero
#   . kernel7.img     32-bit Rpi 2 & Rpi 3
#   . kernel7l.img    32-bit Rpi 4
#   . kernel8.img     64-bit Rpi 3 & Rpi 4
#-------------------------------------------------------------------------------
if(CMAKE_SYSTEM_PROCESSOR STREQUAL "aarch64")
  set(kernel_img_file kernel8.img)
elseif(CMAKE_SYSTEM_PROCESSOR STREQUAL "armv8")
  set(kernel_img_file kernel7.img)
endif()

add_custom_target(
  kernel_img
  ${CMAKE_OBJCOPY} $<TARGET_FILE:kernel> -O binary ${CMAKE_CURRENT_BINARY_DIR}/${kernel_img_file}
  BYPRODUCTS ${CMAKE_CURRENT_BINARY_DIR}/${kernel_img_file}
  DEPENDS kernel
  COMMENT "Making kernel image...")
