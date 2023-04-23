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

The canonical way to map memory is to direct-map all (or most) of the physical
memory into the virtual address space. Since the bootstrap code is not aware of
the amount of physical memory, it takes a conservative approach by mapping only
what is need to get the kernel going. The Rustland code will parse the ATAGs /
DTB, then map appropriately for the architecture.

The minimal page table structure maps the kernel and DTB (if present) into the
kernel's virtual address space using their physical offsets from the virtual
address base.

Additionally, the bootstrap code sets up a parallel identity mapping in the MMU
for the kernel. Since the CPU's program counter will still be using physical
addresses after enabling the MMU, this identity mapping allows the CPU to
continue executing instructions. The bootstrap code using a jump to a virtual
address to switch the program counter over, then clears the identity mapping.

The bootstrap code aligns the size of the kernel area (`__kernel_id_pages_end`)
and the size of the DTB (if present) on 2 MiB boundaries to skip Level 4
address translation.

For example, an AArch64 layout with a 2.5 MiB kernel and 16 KiB DTB on a
Raspberry Pi 3 might look like:

                 Physical Address         Virtual Address
    -----------------------------------------------------------
    Kernel       0x0000_0000_0000_0000-   0xffff_8000_0000_0000-
    Virtual      0x0000_0000_0040_0000    0xffff_8000_0040_0000

    Kernel       0x0000_0000_0000_0000-   0x0000_0000_0000_0000-
    Identity     0x0000_0000_0040_0000    0x0000_0000_0040_0000

    DTB          0x0000_0000_0800_0000-   0xffff_8000_0800_0000-
                 0x0000_0000_0820_0000    0xffff_8000_0820_0000
