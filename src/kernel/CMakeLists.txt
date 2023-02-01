#-------------------------------------------------------------------------------
# Generate the linker script.
#-------------------------------------------------------------------------------
set(ld_in ${CMAKE_CURRENT_LIST_DIR}/${CMAKE_SYSTEM_PROCESSOR}/boot.ld.in)
set(ld_out ${CMAKE_CURRENT_BINARY_DIR}/boot.ld)
add_custom_command(
  OUTPUT ${ld_out}
  COMMAND ${CMAKE_C_COMPILER} -DVIRT_BASE=${ROS_VIRTUAL_BASE_ADDRESS}
                              -DKERNEL_BASE=${ROS_KERNEL_BASE_ADDRESS}
                              -CC -E -P -x c ${ld_in} >${ld_out}
  DEPENDS ${ld_in}
  COMMENT "Making linker script...")

add_custom_target(boot_script DEPENDS ${ld_out})

#-------------------------------------------------------------------------------
# Build the kernel executable.
#-------------------------------------------------------------------------------
add_executable(
  kernel ${CMAKE_SYSTEM_PROCESSOR}/boot.S
         ${CMAKE_SYSTEM_PROCESSOR}/exceptions.S
         ${CMAKE_SYSTEM_PROCESSOR}/pages.S)
add_dependencies(kernel boot_script)
target_include_directories(kernel PRIVATE ${CMAKE_SYSTEM_PROCESSOR}/include)
target_compile_definitions(kernel PRIVATE RPI_VERSION=${RPI_VERSION})
target_link_libraries(kernel PRIVATE ROS::Kernel)
target_link_options(kernel PUBLIC -T ${ld_out})

#-------------------------------------------------------------------------------
# Dump the kernel assembly.
#-------------------------------------------------------------------------------
set(asm_out ${CMAKE_CURRENT_BINARY_DIR}/asm.txt)
add_custom_command(
  OUTPUT ${asm_out}
  COMMAND ${CMAKE_OBJDUMP} -D $<TARGET_FILE:kernel> >${asm_out}
  DEPENDS $<TARGET_FILE:kernel>
  COMMENT "Dumping assembly...")

add_custom_target(kernel_asm ALL DEPENDS ${asm_out})

#-------------------------------------------------------------------------------
# Build the raw binary image file.
#-------------------------------------------------------------------------------
set(image_out ${CMAKE_CURRENT_BINARY_DIR}/${ROS_KERNEL_IMAGE_FILE})
add_custom_command(
  OUTPUT ${image_out}
  COMMAND ${CMAKE_OBJCOPY} $<TARGET_FILE:kernel> -O binary ${image_out}
  DEPENDS $<TARGET_FILE:kernel>
  COMMENT "Making kernel image...")

add_custom_target(kernel_img ALL DEPENDS ${image_out})
