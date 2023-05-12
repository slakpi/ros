add-symbol-file build/Debug/aarch64-none-elf/src/kernel/kernel
target remote localhost:9000
b ros_kernel
