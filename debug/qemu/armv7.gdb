add-symbol-file build/Debug/arm-none-eabi/src/start/armv7/kernel
target remote localhost:9000
b ros_kernel_init
