add-symbol-file build/Debug/aarch64-none-elf/src/start/aarch64/kernel
target remote localhost:9000
b ros_kernel
