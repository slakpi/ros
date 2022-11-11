if(CMAKE_SYSTEM_PROCESSOR STREQUAL "aarch64")
  add_library(kernel_stub STATIC aarch64.S)
  target_link_options(kernel_stub PUBLIC -T ${CMAKE_CURRENT_LIST_DIR}/aarch64.ld)
elseif(CMAKE_SYSTEM_PROCESSOR STREQUAL "armv7")
  add_library(kernel_stub STATIC armv7.S)
  target_link_options(kernel_stub PUBLIC -T ${CMAKE_CURRENT_LIST_DIR}/armv7.ld)
endif()

add_library(Kernel::Stub ALIAS kernel_stub)
