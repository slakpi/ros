Kernel Start
============

This folder contains all of the architecture-dependent assembly to perform the
bare minimum work required to configure the CPU(s) and transfer control to
Rustland.

_start
------

`_start` is the entry point called by the bootloader.

Exceptions
----------

The start code provides thin wrapper exception vectors that gather the exception
information before calling into Rustland.

MMU
---

The canonical way to map memory is to direct-map all (or most) of the physical
memory into the virtual address space. Since the start code is not aware of the
amount of physical memory, it takes a conservative approach by mapping only what
is needed to get the kernel going. The Rustland code will create the final
address space layout.

The minimal page table structure maps the kernel and DTB (if present) into the
kernel's virtual address space using their physical offsets from the virtual
address base.

Initialization
--------------

