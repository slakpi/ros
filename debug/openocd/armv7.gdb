add-symbol-file build/Debug/arm-none-eabi/src/start/armv7/kernel
target extended-remote :3333
b ros_kernel_init
