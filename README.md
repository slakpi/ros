ROS
===

Introduction
------------

ROS... Call it "Rust OS," "Raspberry Pi OS," "Randy OS," whatever. It is not a
unique acronym among Rust kernel/OS projects. This project is an interpretation
of several Raspberry Pi bare metal C programming tutorials written in Rust with
some assembly start code, of course. See the `References` section for a list of
tutorial links.

Building
--------

ROS requires CMake 3.19 or higher and includes Corrosion as a submodule. The GCC
and Rust ARM toolchains are required to do the actual compilation. ROS includes
CMake toolchain files and scripts to do most of the configuration for AArch64 or
ARMv7. The scripts also handle Raspberry Pi specific configuration.

ROS is currently setup to support AArch64 for Raspberry Pi 2 (rev. 1.2), 3, and
4 boards; and ARMv7 for Raspberry Pi 2 (rev. 1.1) boards. Typical CMake
configuration commands are:

    cmake -B build/aarch64 \
          -G Ninja \
          -DRPI_VERSION=3 \
          -DQEMU_BUILD=True \
          -DCMAKE_TOOLCHAIN_FILE=cmake/aarch64-none-elf.cmake \
          -DTC_PATH=$HOME/.local/cross \
          -DCMAKE_BUILD_TYPE=Debug .

or:

    cmake -B build/armv7 \
          -G Ninja \
          -DRPI_VERSION=2 \
          -DQEMU_BUILD=True \
          -DCMAKE_TOOLCHAIN_FILE=cmake/arm-none-eabi.cmake \
          -DTC_PATH=$HOME/.local/cross \
          -DCMAKE_BUILD_TYPE=Debug .

These assume:

* Ninja as the build tool
* The official ARM GCC toolchains are installed under:
  * AArch64: `${TC_PATH}/gnu-aarch64-none-elf`
  * ARM: `${TC_PATH}/gnu-arm-none-eabi`
* The appropriate Rust toolchain installed in a discoverable location.

`rustup` is highly recommended for installing the Rust toolchains. The AArch64
build requires the `aarch64-unknown-none` Rust toolchain and the ARMv7 build
requires the `armv7a-none-eabi` toolchain.

Building is done simply by calling:

    cmake --build build/aarch64

or:

    cmake --build build/armv7

The build will generate a kernel image appropriate to the platform. Currently:

* AArch64: `kernel8.img`
* ARMv7 (Raspberry Pi 4): `kernel7l.img`
* ARMv7 (Raspberry Pi 2 or 3): `kernel7.img`
* Unknown Platform: `kernel.img`

Visual Studio Code
------------------

The `.vscode` directory contains a CMake configuration file for Visual Studio
Code's CMake extensions. The README in that folder briefly explains how to setup
a local `settings.json` file to simplify building ROS with Code.

Running with QEMU
-----------------

The `QEMU_BUILD` variable must be set to `True` when building to run the kernel
in QEMU. This flag instructs platform configuration scripts to adjust the kernel
base address as necessary for QEMU. For example, QEMU's Raspberry Pi 2 profile
expects the kernel at `0x10000` rather than `0x8000`.

Start QEMU with:

    qemu-system-aarch64 -M raspi3b \
                        -kernel build/aarch64/src/boot/kernel8.img \
                        -serial null -serial stdio \
                        -gdb tcp::9000 \
                        -S

or:

    qemu-system-arm -M raspi2b \
                    -kernel build/armv7/src/boot/kernel7.img \
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