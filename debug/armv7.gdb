add-symbol-file build/Debug/arm-none-eabi/src/bootstrap/kernel
target remote localhost:9000
b ros_kernel
