#-------------------------------------------------------------------------------
# AArch64 kernel start code.
#-------------------------------------------------------------------------------
add_executable(
  kernel dtb.S
         exceptions.S
         pages.S
         spin_lock.S
         start.S)
target_include_directories(kernel PRIVATE ${CMAKE_CURRENT_LIST_DIR}/include)
