# ROS KERNEL ARCHITECTURE

## Table of Contents

* [`start` Library](#start-library)
  * [ARMv7 Startup](#armv7-startup)
  * [AArch64 Startup](#aarch64-startup)
* [`ros_kernel` Library](#ros_kernel-library)
* [`ros_kernel_user` Library](#ros_kernel_user-library)
* [`ros_user` Library](#ros_user-library)
* [Reference](#reference)

## `start` Library {#start-library}

The `start` library is the low-level entry point for the kernel and is currently implemented for the [ARM and Aarch64 Linux Boot Protocols](#reference).

### ARMv7 Startup

#### Page size {#armv7-page-size}

#### Kernel Image Layout {#armv7-kernel-image-layout}

#### Operating Mode

#### Basic Startup {#armv7-basic-startup}

#### Initial Page Tables {#armv7-initial-page-tables}

#### Transfer to Kernel Initialization {#armv7-xfer-to-kernel-init}

### AArch64 Startup

#### Page Size {#aarch64-page-size}

`__page_size` is a compile-time constant provided by the linker script that specifies the size of a page. The CMake build system enforces valid AArch64 page sizes, but, currently, the kernel will intentionally panic if the page size is not 4 KiB.

#### Kernel Image Layout {#aarch64-kernel-image-layout}

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

The base of the `.text` segment is specified by the compile-time constant `KERNEL_BASE` provided by CMake.

`.data.stack` is the primary core's interrupt service routine (ISR) stack. Refer to `SP_EL1`. `__kernel_stack_pages` is a compile-time constant provided by the linker script that specifies the ISR stack size in pages. During the single-threaded setup phase, the primary core uses this stack as its general purpose stack.

`.data.stack_pointers` is the ISR stack pointer table for secondary cores. During the single-threaded setup phase, the primary core will allocate pages for secondary core ISR stacks and place pointers to the tops of those stacks in this table. The secondary cores will index this table to locate their stacks when they are released.

The stack pointer table is a single page of 512 8-byte pointer entries. 512 entries is sufficient for the 256 core maximum on AArch64 nodes.

`.data.id_pages` and `.data.pages` are blocks reserved for the initial kernel page tables. The kernel image reserves three pages for each table.

#### Exception Level

The boot loader will have already put the primary core into EL2 or EL1. On startup, ROS ensures the primary core is in EL1 before performing startup tasks.

#### Basic Startup {#aarch64-basic-startup}

Once in EL1 on the primary core, ROS sets the primary core's stack pointer to `__kernel_stack_start` so that it can start calling helper functions using the [AArch64 procedure call standard](#reference).

With the stack set, ROS writes all zeros to the `.bss` section.

Next, ROS checks if the blob provided by the boot loader is a DeviceTree by checking if the first four bytes are the DeviceTree magic bytes.

#### Initial Page Tables {#aarch64-initial-page-tables}

`__virtual_start` is a compile-time constant provided by the linker script specifying the virtual address base of the kernel.

Because ROS has no idea how much memory actually exists in the system at this point, it takes a very conservative approach to the initial page tables. The kernel image and the DeviceTree binary (DTB), if present, are linearly mapped in 2 MiB sections. The identity tables map the physical addresses back to the same physical address while the virtual address page tables map the physical addresses offset by `__virtual_start`.

Each table has three pages, one for each fo the L1, L2, and L3 tables. Only the first entries of the L1 and L2 tables are used for the first 1 GiB of the virtual address space. The 2 MiB sections of the kernel image and DTB are mapped in the L3 table.

`E` is the end address of the kernel image `__kernel_start + __kernel_size` rounded to the next 2 MiB section, `P` is the blob pointer provided by the boot loader, and `D` is the DTB end address rounded to the next 2 MiB section.

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

The identity tables allow a core to find the next instruction, typically a jump to set the program counter to virtual addressing, after enabling the MMU. After making the jump to virtual addressing, ROS sets `TTBR0_EL1` to 0.

The identity tables are placed in the kernel image prior to the virtual tables to ensure they remain intact for the secondary cores.

#### Transfer to Kernel Initialization {#aarch64-xfer-to-kernel-init}

After enabling the MMU, the primary core fills out the AArch64 kernel configuration struct and passes it to `ros_kernel_init` in the `ros_kernel` library. All addresses in the struct are physical.

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

## `ros_kernel` Library

### `arch` Module

The `arch` module is an *interface* to architecture-specific Rust code. The module automatically includes the correct architecture code and exports it as the `arch` module.

#### Module Interface {#arch-module-iface}

Each architecture supported by ROS must implement the following public interface.

##### `pub fn arch::init( config: usize )`

Performs single-threaded, architecture-specific kernel initialization. Typically, this will involve determining the amount of physical memory, setting up kernel page tables, setting up page allocators, etc.

##### `pub fn arch::init_secondary_cores()`

Performs secondary core initialization. All secondary cores should be running with interrupts disabled when this function returns.

##### `pub fn get_memory_layout() -> &'static memory::MemoryConfig`

Retrieves the physical memory layout.

##### `pub fn get_exclusion_layout() -> &'static memory::MemoryConfig`

Retrieves the physical memory exclusions. These are areas that will not be made available to the page allocators. For example, the area containing the kernel image must be excluded.

##### `pub fn get_page_size() -> usize`

Retrieves the page size.

##### `pub fn get_page_shift() -> usize`

Retrieves the number of bits to shift to shift an address right to calculate a physical Page Frame Number (PFN).

##### `pub fn get_kernel_base() -> usize`

Retrieves the kernel's physical base address.

##### `pub fn get_kernel_virtual_base() -> usize`

Retrieves the kernel's virtual base address.

##### `pub fn get_max_physical_address() -> usize`

Retrieves the maximum physical address.

##### `pub fn get_core_count() -> usize`

Retrieves the number of cores available on this node.

##### `pub fn get_core_info( core: usize ) -> cpu::CoreInfo`

Retrieves architecture-independent information about a core.

##### `pub fn get_core_id() -> usize`

Retrieves the identifier of the current core.

##### `pub fn spin_lock( lock_addr: usize )`

Low-level spin lock on the specified address.

##### `pub fn try_spin_lock( lock_addr: usize ) -> bool`

Attempt a low-level spin lock on the specified address.

##### `pub fn spin_unlock( lock_addr: usize )`

Low-level spin lock release on the specified address.

##### `pub fn debug_print( args: fmt::Arguments )`

Implements architecture-dependent debug output. For example, ROS currently uses the ARM UART to send debug messages.

#### ARMv7 {#armv7-arch-impl}

##### Memory Initialization {#armv7-memory-init}

##### Address Space {#armv7-address-space}

ROS uses a 3:1 split configuration with a fixed, linear mapping to the first 896 MiB of physical memory in the Low Memory area of the kernel segment.

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

The 1 MiB Thread Local area is reserved for mapping per-thread temporary page tables to access the upper 2,176 MiB of physical memory. With 4 KiB pages and a maximum core count of 256, 1 MiB allows each core to have its own mapping for the thread it is currently running.

The 32 MiB Page Directory area is a virtually-contiguous array of page metadata entries. With 4 KiB pages, the 4 GiB address space has 1 Mi pages. 32 MiB allows for 32 bytes of metadata for each page.

Why 32 bytes? Will we need more? Great questions! Anyway...

Similar to the Linux sparse virtual memory map model, this simplifies conversion from a page metadata address to a page physical address and vice versa. For 4 KiB pages:

    Page Frame Number (PFN) = Physical Address >> 12
    Page Metadata Address   = ( PFN << 5 ) + 0xf810_0000

The process is easily reversed to calculate a page physical address from a page metadata address.

ROS configures ARMv7 cores to place exception vectors at 0xffff_0000 and places the stub pointers in the following page at 0xffff_10000. The top 56 KiB of the address space are unused.

The remaining 97,216 KiB of the kernel segment are available for...things.

##### CPU Initialization {#armv7-cpu-init}

#### AArch64 {#aarch64-arch-impl}

##### Memory Initialization {#aarch64-memory-init}

##### Address Space {#aarch64-address-space}

ROS uses the conventional 256 TiB arrangement for a 64-bit address space and allows up to 254 TiB of physical memory accessed through a fixed, linear mapping.

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

The 2 TiB Page Directory area is a virtually-contiguous array of page metadata entries. With 4 KiB pages, the 256 TiB address space has 64 Gi pages. 2 TiB allows for 32 bytes of metadata for each page.

Why 32 bytes? Will we need more? Great questions! Anyway...

Similar to the Linux sparse virtual memory map model, this simplifies conversion from a page metadata address to a page physical address and vice versa. For 4 KiB pages:

    Page Frame Number (PFN) = Physical Address >> 12
    Page Metadata Address   = ( PFN << 5 ) + 0xffff_fffe_0000_0000

The process is easily reversed to calculate a page physical address from a page metadata address.

The exception vectors are part of the kernel image.

##### CPU Initialization {#aarch64-cpu-init}

### `mm` Module

#### Pager

#### Page Directory

#### Buddy Allocator

Refer to [Buddy Allocator](#reference).

A buddy allocator manages a single contiguous block of memory and allocates blocks of up to 2^10 pages. The buddy allocator has a small amount of overhead to track buddy pair state. The allocator computes the size buddy pair state from the size of the memory block, rounds up to the nearest page, and stores the state at the end of the memory block.

    Block Start                                  End
    +--------------------------------------+-------+
    | Available Pages                      | State |
    +--------------------------------------+-------+

On a system with 1 GiB of physical memory and 4 KiB pages, the buddy allocator needs just shy of 32 KiB for the buddy pair state. Out of the 256 Ki pages available, the buddy allocator will reserve 8 of them for the overhead.

During initialization, the buddy allocator embeds a linked list of free pages for each order directly into the pages themselves.

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

The checksum is a simple XOR of the architecture checksum seed and the next and previous pointers. It is not a secure checksum, it is only meant as a sanity check when allocating a page.

### `support` Module

### `sync` Module

## `ros_kernel_user` Library

## `ros_user` Library

## Reference

* [Linux ARM Boot Protocol](https://www.kernel.org/doc/Documentation/arm/booting.rst)
* [Linux AArch64 Boot Protocol](https://www.kernel.org/doc/Documentation/arm64/booting.txt)
* [AArch32 Procedure Call Standard](https://github.com/ARM-software/abi-aa/blob/main/aapcs32/aapcs32.rst)
* [AArch64 Procedure Call Standard](https://github.com/ARM-software/abi-aa/blob/main/aapcs64/aapcs64.rst)
* [Linux Memory Models](https://lwn.net/Articles/789304/)
* [Buddy Memory Allocation](https://en.wikipedia.org/wiki/Buddy_memory_allocation)
