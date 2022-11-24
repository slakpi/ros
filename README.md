ROS
===

Introduction
------------

ROS... Call it "Rust OS," "Raspberry Pi OS," "Randy OS," whatever. This project
is an interpretation of several Raspberry Pi bare metal C programming tutorials
written in Rust...with some ARM assembly and C bootstrapping, of course. See the
`References` section for a list of tutorial links.

Building
--------

ROS requires CMake 3.19 or higher and includes Corrosion as a submodule. The GCC
and Rust ARM toolchains are required to do the actual compilation. ROS includes
two CMake toolchain files to do most of the configuration for AArch64 or ARMv7.

ROS is currently setup to support AArch64 for Raspberry Pi 3 and 4 boards, and
ARMv7 for Raspberry Pi 2 boards. Typical CMake configuration commands are:

    cmake -B build/aarch64 \
          -G Ninja \
          -DRPI_VERSION=3 \
          -DQEMU_BUILD=True \
          -DCMAKE_TOOLCHAIN_FILE=cmake/aarch64-none-elf.cmake \
          -DTC_PATH=$HOME/.local/cross/gnu-aarch64-none-elf \
          -DCMAKE_BUILD_TYPE=Debug .

or:

    cmake -B build/armv7 \
          -G Ninja \
          -DRPI_VERSION=2 \
          -DQEMU_BUILD=True \
          -DCMAKE_TOOLCHAIN_FILE=cmake/arm-none-eabi.cmake \
          -DTC_PATH=$HOME/.local/cross/gnu-arm-none-eabi \
          -DCMAKE_BUILD_TYPE=Debug .

These assume Ninja as the build tool, the official ARM GCC toolchainsinstalled
in the path specified for `TC_PATH`, and the appropriate Rust toolchain
installed in a discoverable location.

`rustup` is highly recommended for installing the Rust toolchains. The AArch64
build requires the `aarch64-unknown-none` Rust toolchain and the ARMv7 build
requires the `armv7a-none-eabi` toolchain.

Building is done simply by calling:

    cmake --build build/aarch64

or:

    cmake --build build/armv7

The build will create a file named `kernel` that is the executable and the raw
kernel binary, either: `kernel7.img` (ARMv7) or `kernel8.img` (AArch64).

Running with QEMU
-----------------

The `QEMU_BUILD` variable must be set to `True` when building to run the kernel
in QEMU. Technically, for AArch64 it doesn't matter. However, the QEMU profile
for the Raspberry Pi 2 expects the kernel at 0x10000 rather than 0x8000 like the
actual hardware. Start QEMU with:

    qemu-system-aarch64 -M raspi3b \
                        -kernel build/aarch64/src/arch/kernel8.img \
                        -serial null -serial stdio \
                        -gdb tcp::9000 \
                        -S

or:

    qemu-system-arm -M raspi2b \
                    -kernel build/armv7/src/arch/kernel7.img \
                    -serial null -serial stdio \
                    -gdb tcp::9000 \
                    -S

Note: `-serial null -serial stdio` sets UART0 to null and UART1 to `stdio`. The
mini UART on the Raspberry Pi will output to UART1.

Note: `-gdb tcp::9000 -S` sets QEMU up to halt immediately and listen for a GNU
debugger connection on TCP port 9000. The `debug` folder has two GDB scripts to
connect to QEMU and load symbols for the kernel. For example:

    gdb -x debug/aarch64.gdb

will connect to QEMU running the AArch64 kernel and load AArch64 `kernel` file
for symbols.

References
----------

  * https://github.com/s-matyukevich/raspberry-pi-os
  * https://github.com/isometimes/rpi4-osdev
  * https://www.youtube.com/channel/UCRWXAQsN5S3FPDHY4Ttq1Xg/playlists
  * https://www.cl.cam.ac.uk/projects/raspberrypi/tutorials/os/index.html
  * https://os.phil-opp.com/
  * https://wiki.osdev.org/Main_Page
  * https://github.com/corrosion-rs/corrosion
  * https://developer.arm.com/downloads/-/arm-gnu-toolchain-downloads
  * https://rustup.rs/