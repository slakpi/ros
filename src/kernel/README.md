Kernel Bootstrap
================

This folder contains all of the architecture-dependent code to bootstrap the
kernel. It provides the entry point called by the bootloader, initializes the
processor, and transfers control to Rustland.

_start
------

`_start` is the entry point called by the bootloader. `_start` halts any other
running instances on other CPUs, moves the kernel into the appropriate
protection ring, sets up exception vectors, then transfers control to
`kernel_stub` in Rustland.

Exceptions
----------

The bootstrap code provides thin wrapper exception vectors that gather the
exception information before calling into Rustland.
