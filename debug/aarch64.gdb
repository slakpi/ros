add-symbol-file build/aarch64/src/kernel/kernel
target remote localhost:9000
b ros_kernel
