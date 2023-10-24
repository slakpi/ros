include(platform)

#-------------------------------------------------------------------------------
# Generate the linker script by using the C pre-processor to replace literal
# definitions in the `boot.ld.in` file.
#   -E    Pre-process only.
#   -P    Inhibit line markers (not calling on C source code)
#   -x c  Treat as C source code.
#-------------------------------------------------------------------------------
get_kernel_base_address(kernel_base)
get_kernel_virtual_base_address(kernel_virtual_base kernel_vmsplit)
get_kernel_stack_pages(kernel_stack_pages)
get_kernel_page_size(kernel_page_size)
set(ld_in ${CMAKE_CURRENT_LIST_DIR}/${CMAKE_SYSTEM_PROCESSOR}/boot.ld.in)
set(ld_out ${CMAKE_CURRENT_BINARY_DIR}/boot.ld)
add_custom_command(
  OUTPUT ${ld_out}
  COMMAND ${CMAKE_C_COMPILER} -DKERNEL_VIRTUAL_BASE=${kernel_virtual_base}
                              -DKERNEL_VMSPLIT=${kernel_vmsplit}
                              -DKERNEL_BASE=${kernel_base}
                              -DKERNEL_STACK_PAGES=${kernel_stack_pages}
                              -DKERNEL_PAGE_SIZE=${kernel_page_size}
                              -E -P -x c ${ld_in} >${ld_out}
  DEPENDS ${ld_in}
  COMMENT "Making linker script...")

add_custom_target(ld_script DEPENDS ${ld_out})

#-------------------------------------------------------------------------------
# Build the kernel executable. Link using the generated linker script.
#-------------------------------------------------------------------------------
add_subdirectory(${CMAKE_SYSTEM_PROCESSOR})
add_dependencies(kernel ld_script)
target_link_libraries(kernel PRIVATE ROS::Kernel)
target_link_options(kernel PUBLIC -T ${ld_out})
target_arch_agnostic_asm_options(kernel)
target_arch_asm_options(kernel)

#-------------------------------------------------------------------------------
# Build the raw binary image file.
#-------------------------------------------------------------------------------
get_kernel_image_file(image_file)
set(image_out ${CMAKE_CURRENT_BINARY_DIR}/${image_file})
add_custom_command(
  OUTPUT ${image_out}
  COMMAND ${CMAKE_OBJCOPY} $<TARGET_FILE:kernel> -O binary ${image_out}
  DEPENDS $<TARGET_FILE:kernel>
  COMMENT "Making kernel image...")

add_custom_target(kernel_img ALL DEPENDS ${image_out})

#-------------------------------------------------------------------------------
# Dump the kernel assembly. `dummy_asm_out` is a hack that specifies a file that
# is never created. This forces CMake to run the custom command every time.
#-------------------------------------------------------------------------------
set(asm_out ${CMAKE_CURRENT_BINARY_DIR}/asm.txt)
set(dummy_asm_out ${CMAKE_CURRENT_BINARY_DIR}/__asm.txt)
add_custom_command(
  OUTPUT ${asm_out} ${dummy_asm_out}
  COMMAND ${CMAKE_OBJDUMP} -D $<TARGET_FILE:kernel> >${asm_out}
  DEPENDS kernel_img
  COMMENT "Dumping kernel assembly...")

add_custom_target(kernel_img_with_asm DEPENDS ${dummy_asm_out})
