add-symbol-file build/Debug/arm-none-eabi/src/bootstrap/armv7/kernel
target remote localhost:9000
b ros_kernel
