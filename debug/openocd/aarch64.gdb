add-symbol-file build/Debug/aarch64-none-elf/src/start/aarch64/kernel
target extended-remote :3333
b ros_kernel_init
