#-------------------------------------------------------------------------------
# ARMv7a kernel bootstrap code.
#-------------------------------------------------------------------------------
add_executable(
  kernel boot.S
         dtb.S
         exceptions.S
         pages_lpae.S
         pages.S)
target_include_directories(kernel PRIVATE ${CMAKE_CURRENT_LIST_DIR}/include)
