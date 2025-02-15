# ROS KERNEL ARCHITECTURE

## Table of Contents

* [`start` Library](#a-start-library)
  * [ARMv7 Startup](#a1-armv7-startup)
  * [AArch64 Startup](#a2-aarch64-startup)
* [`ros_kernel` Library](#b-ros_kernel-library)
* [`ros_kernel_user` Library](#c-ros_kernel_user-library)
* [`ros_user` Library](#d-ros_user-library)
* [Reference](#e-reference)

## A. `start` Library

The `start` library is the low-level entry point for the kernel and is currently implemented for the
ARM and AArch64 boot protocols (See the [Reference](#e-reference) section).

### A.1 ARMv7 Startup

### A.2 AArch64 Startup

#### A.2.1 Page Size

`__page_size` is a compile-time constant provided by the linker script that specifies the size of a
page. The CMake build system enforces valid AArch64 page sizes, but, currently, the kernel will
intentionally panic if the page size is not 4 KiB.

#### A.2.2 Kernel Image Layout

    +----------------------+ __text_start
    | .text                |
    +----------------------+ __rodata_start
    | .rodata              |
    +----------------------+ __data_start
    | .data                |
    +----------------------+ __bss_start
    | .bss                 |
    +----------------------+ __kernel_stack_end
    | .data.stack          | 
    +----------------------+ __kernel_stack_list
    | .data.stack_pointers |
    +----------------------+ __kernel_id_pages_start
    | .data.id_pages       |
    +----------------------+ __kernel_pages_start
    | .data.pages          |
    +----------------------+ __kernel_end

The base of the `.text` segment is specified by the compile-time constant `KERNEL_BASE` provided by
CMake.

`.data.stack` is the primary core's interrupt service routine (ISR) stack. Refer to `SP_EL1`.
`__kernel_stack_pages` is a compile-time constant provided by the linker script that specifies the
ISR stack size in pages. During the single-threaded setup phase, the primary core uses this stack as
its general purpose stack.

`.data.stack_pointers` is the ISR stack pointer table for secondary cores. During the single-
threaded setup phase, the primary core will allocate pages for secondary core ISR stacks and place
pointers to the tops of those stacks in this table. The secondary cores will index this table to
locate their stacks when they are released.

The stack pointer table is a single page of 512 8-byte pointer entries. 512 entries is sufficient
for the 256 core maximum on AArch64 nodes.

`.data.id_pages` and `.data.pages` are blocks reserved for the initial kernel page tables. The
kernel image reserves three pages for each table.

#### A.2.3 Exception Level

The boot loader will have already put the primary core into EL2 or EL1. On startup, ROS ensures the
primary core is in EL1 before performing startup tasks.

#### A.2.4 Basic Startup

Once in EL1 on the primary core, ROS sets the primary core's stack pointer to `__kernel_stack_start`
so that it can start calling helper functions using the AArch64 procedure call standard (See the
[Reference](#e-reference) section).

With the stack set, ROS writes all zeros to the `.bss` section.

Next, ROS checks if the blob provided by the boot loader is a DeviceTree by checking if the first
four bytes are the DeviceTree magic bytes.

#### A.2.5 Initial Page Tables

`__virtual_start` is a compile-time constant provided by the linker script specifying the virtual
address base of the kernel.

Because ROS has no idea how much memory actually exists in the system at this point, it takes a very
conservative approach to the initial page tables. The kernel image and the DeviceTree binary (DTB),
if present, are linearly mapped in 2 MiB sections. The identity tables map the physical addresses
back to the same physical address while the virtual address page tables map the physical addresses
offset by `__virtual_start`.

Each table has three pages, one for each fo the L1, L2, and L3 tables. Only the first entries of the
L1 and L2 tables are used for the first 1 GiB of the virtual address space. The 2 MiB sections of
the kernel image and DTB are mapped in the L3 table.

`E` is the end address of the kernel image `__kernel_start + __kernel_size` rounded to the next 2
MiB section, `P` is the blob pointer provided by the boot loader, and `D` is the DTB end address
rounded to the next 2 MiB section.

                       Identity              Virtual
                       Map                   Map

                 D +---------------+     +---------------+ __virtual_start + D
                   | DTB           |     | DTB           |
                 P +---------------+     +---------------+ __virtual_start + P
                   | / / / / / / / |     | / / / / / / / |
                   | / / / / / / / |     | / / / / / / / |
                 E +---------------+     +---------------+ __virtual_start + E
                   |               |     |               |
                   | Kernel Image  |     | Kernel Image  |
                   |               |     |               |
    __kernel_start +---------------+     +---------------+ __virtual_start + __kernel_start
                   | / / / / / / / |     | / / / / / / / |
                 0 +---------------+     +---------------+ __virtual_start

The identity tables allow a core to find the next instruction, typically a jump to set the program
counter to virtual addressing, after enabling the MMU. After making the jump to virtual addressing,
ROS sets `TTBR0_EL1` to 0.

The identity tables are placed in the kernel image prior to the virtual tables to ensure they remain
intact for the secondary cores.

#### A.2.6 Transfer to Kernel Initialization

After enabling the MMU, the primary core fills out the AArch64 kernel configuration struct and
passes it to `ros_kernel_init` in the `ros_kernel` library. All addresses in the struct are
physical.

    +------------------------------+ 0
    | Virtual base address         |
    +------------------------------+ 8
    | Page size                    |
    +------------------------------+ 16
    | Physical blob address        |
    +------------------------------+ 24
    | Physical kernel address      |
    +------------------------------+ 32
    | Kernel size                  |
    +------------------------------+ 40
    | Physical page tables address |
    +------------------------------+ 48
    | Page table area size         |
    +------------------------------+ 56
    | ISR stack list address       |
    +------------------------------+ 64
    | ISR stack page count         |
    +------------------------------+ 72
    | / / / / / / / / / / / / / /  |
    +------------------------------+ 80

## B. `ros_kernel` Library

### B.1 `arch` Module

The `arch` module is an *interface* to architecture-specific Rust code. The module automatically
includes the correct architecture code an exports it as the `arch` module.

#### B.1.1 ARMv7

##### B.1.1.1 Memory Initialization

##### B.1.1.2 Address Space

ROS uses a 3:1 split configuration with a fixed, linear mapping to the first 896 MiB of physical
memory in the Low Memory area of the kernel segment.

    +-----------------+ 0xffff_ffff    -+
    | / / / / / / / / | 56 KiB (Unused) |
    |.................| 0xffff_2000     |
    | Exception Stubs | 4 KiB           |                  K
    |.................| 0xffff_1000     |                  E
    | Vectors         | 4 KiB           |                  R
    |.................| 0xffff_0000     |                  N 
    |                 |                 +- High Memory     E
    |                 | 97,216 KiB      |                  L
    |                 |                 |
    |.................| 0xfa10_0000     |                  S
    | Page Directory  | 32 MiB          |                  E
    |.................| 0xf810_0000     |                  G
    | Thread Local    | 1 MiB           |                  M
    +-----------------+ 0xf800_0000    -+                  E
    |                 |                 |                  N
    |                 |                 |                  T
    | Fixed Mappings  | 896 MiB         +- Low Memory
    |                 |                 |
    |                 |                 |
    +-----------------+ 0xc000_0000    -+
    |                 |
    |                 |
    | User Segment    | 3 GiB
    |                 |
    |                 |
    +-----------------+ 0x0000_0000

The 1 MiB Thread Local area is reserved for mapping per-thread temporary page tables to access the
upper 2,176 MiB of physical memory. With 4 KiB pages and a maximum core count of 256, 1 MiB allows
each core to have its own mapping for the thread it is currently running.

The 32 MiB Page Directory area is a virtually-contiguous array of page metadata entries. With 4 KiB
pages, the 4 GiB address space has 1 Mi pages. 32 MiB allows for 32 bytes of metadata for each page.

Why 32 bytes? Will we need more? Great questions! Anyway...

Similar to the Linux sparse virtual memory map model, this simplifies conversion from a page
metadata address to a page physical address and vice versa. For 4 KiB pages:

    Page Frame Number (PFN) = Physical Address >> 12
    Page Metadata Address   = ( PFN << 5 ) + 0xf810_0000

The process is easily reversed to calculate a page physical address from a page metadata address.

ROS configures ARMv7 cores to place exception vectors at 0xffff_0000 and places the stub pointers in
the following page at 0xffff_10000. The top 56 KiB of the address space are unused.

The remaining 97,216 KiB of the kernel segment are available for...things.

##### B.1.1.3 CPU Initialization

#### B.1.2 AArch64

##### B.1.2.1 Memory Initialization

##### B.1.2.2 Address Space

ROS uses the conventional 256 TiB arrangement for a 64-bit address space and allows up to 254 TiB of
physical memory accessed through a fixed, linear mapping.

    +-----------------+ 0xffff_ffff_ffff_ffff            K S
    | Page Directory  | 2 TiB                            E E
    |.................| 0xffff_fe00_ffff_ffff            R G
    |                 |                                  N M
    | Fixed Mappings  | 254 TiB                          E E
    |                 |                                  L N
    +-----------------+ 0xffff_0000_0000_0000              T
    | / / / / / / / / |
    | / / / / / / / / |
    | / / / / / / / / | 16,776,704 TiB (Unused)
    | / / / / / / / / |
    | / / / / / / / / |
    +-----------------+ 0x0000_ffff_ffff_ffff
    |                 |
    | User Segment    | 256 TiB
    |                 |
    +-----------------+ 0x0000_0000_0000_0000

The 2 TiB Page Directory area is a virtually-contiguous array of page metadata entries. With 4 KiB
pages, the 256 TiB address space has 64 Gi pages. 2 TiB allows for 32 bytes of metadata for each
page.

Why 32 bytes? Will we need more? Great questions! Anyway...

Similar to the Linux sparse virtual memory map model, this simplifies conversion from a page
metadata address to a page physical address and vice versa. For 4 KiB pages:

    Page Frame Number (PFN) = Physical Address >> 12
    Page Metadata Address   = ( PFN << 5 ) + 0xffff_fffe_0000_0000

The process is easily reversed to calculate a page physical address from a page metadata address.

The exception vectors are part of the kernel image.

##### B.1.2.3 CPU Initialization

### B.2 `mm` Module

#### B.2.1 Module Interface

#### B.2.2 Pager

#### B.2.3 Page Directory

#### B.2.4 Buddy Allocator

See: [Buddy Memory Allocation](https://en.wikipedia.org/wiki/Buddy_memory_allocation)

A buddy allocator manages a single contiguous block of memory and allocates blocks of up to 2^10
pages. The buddy allocator has a small amount of overhead to track buddy pair state. The allocator
computes the size buddy pair state from the size of the memory block, rounds up to the nearest page,
and stores the state at the end of the memory block.

    Block Start                                  End
    +--------------------------------------+-------+
    | Available Pages                      | State |
    +--------------------------------------+-------+

On a system with 1 GiB of physical memory and 4 KiB pages, the buddy allocator needs just shy of
32 KiB for the buddy pair state. Out of the 256 Ki pages available, the buddy allocator will reserve
8 of them for the overhead.

During initialization, the buddy allocator embeds a linked list of free pages for each order
directly into the pages themselves.

    +-------------------+ 0
    | Next Pointer      |
    +-------------------+
    | Previous Pointer  |
    +-------------------+
    | Checksum          |
    +-------------------+
    | / / / / / / / / / |
    | / / / / / / / / / |
    | / / / / / / / / / |
    +-------------------+ Page Size

The checksum is a simple XOR of the architecture checksum seed and the next and previous pointers.
It is not a secure checksum, it is only meant as a sanity check when allocating a page.

### B.3 `support` Module

### B.4 `sync` Module

## C. `ros_kernel_user` Library

## D. `ros_user` Library

## E. Reference

* [Linux ARM Boot Protocol](https://www.kernel.org/doc/Documentation/arm/booting.rst)
* [Linux AArch64 Boot Protocol](https://www.kernel.org/doc/Documentation/arm64/booting.txt)
* [AArch32 Procedure Call Standard](https://github.com/ARM-software/abi-aa/blob/main/aapcs32/aapcs32.rst)
* [AArch64 Procedure Call Standard](https://github.com/ARM-software/abi-aa/blob/main/aapcs64/aapcs64.rst)
* [Linux Memory Models](https://lwn.net/Articles/789304/)
