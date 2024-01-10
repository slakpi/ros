add-symbol-file build/Debug/arm-none-eabi/src/start/armv7/kernel
target remote localhost:3333
b ros_kernel
