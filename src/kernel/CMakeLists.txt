add_executable(kernel kernel_main.c)
target_link_libraries(kernel PRIVATE Kernel::Stub)
