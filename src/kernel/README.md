Kernel Bootstrap
================

This folder contains all of the architecture-dependent code to bootstrap the
kernel. It provides the entry point called by the bootloader, initializes the
processor, and transfers control to Rustland.

For now, it is easier to have the ARM GNU toolchain handle assembling the boot-
strap code and linking the ROS Kernel Library in to the final kernel image.

_start
------

`_start` is the entry point called by the bootloader. `_start` halts any other
running instances on other CPUs, moves the kernel into the appropriate
protection ring, sets up exception vectors, sets up the MMU, then transfers
control to `ros_kernel` in Rustland.

Exceptions
----------

The bootstrap code provides thin wrapper exception vectors that gather the
exception information before calling into Rustland.

MMU
---

The bootstrap code sets up the bare minimum page table structure to allow
enabling the MMU before calling into Rustland. The minimal page table structure
maps the kernel, DTB (if present), and peripheral addresses into the kernel's
virtual address space using their physical offsets from the virtual address
base. For example, the bootstrap setup for AArch64 on a Raspberry Pi 3 is:

                 Physical Address         Virtual Address
    -----------------------------------------------------------
    Kernel       0x0000_0000_0000_0000    0xffff_8000_0000_0000
    DTB          0x0000_0000_0800_0000    0xffff_8000_0800_0000
    Peripherals  0x0000_0000_3f00_0000    0xffff_8000_3f00_0000

The canonical way to map memory is to direct-map all (or most) of the physical
memory into the virtual address space. Since the bootstrap code is not aware of
the amount of physical memory, it takes a conservative approach by mapping only
what is need to get the kernel going.

The Rustland code will parse the ATAGs / DTB, then map appropriately for the
architecture.
