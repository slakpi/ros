#-------------------------------------------------------------------------------
# ARMv7a kernel start code.
#-------------------------------------------------------------------------------
add_executable(
  kernel dtb.S
         exceptions.S
         extensions.S
         pages.S
         pages.S
         start.S)
target_include_directories(kernel PRIVATE ${CMAKE_CURRENT_LIST_DIR}/include)
