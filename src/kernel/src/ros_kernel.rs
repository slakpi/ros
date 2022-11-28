use crate::dbg_print;
use crate::peripherals;
use core::panic::PanicInfo;

const MAX_MEM_REGIONS: usize = 16;

/// @struct ROSMemoryRegion
/// @brief Memory region available to the kernel.
/// @var base The base address of the region.
/// @var size The size of the region in bytes.
#[derive(Copy, Clone)]
pub struct ROSMemoryRegion {
  pub base: usize,
  pub size: usize,
}

impl ROSMemoryRegion {
  pub fn new() -> Self {
    ROSMemoryRegion { base: 0, size: 0 }
  }
}

/// @struct ROSKernelInit
/// @brief Initialization parameters provided by the bootloader.
/// @var peripheral_base The base address for peripherals.
pub struct ROSKernelInit {
  pub peripheral_base: usize,
  pub memory_regions: [ROSMemoryRegion; MAX_MEM_REGIONS],
}

impl ROSKernelInit {
  pub fn new() -> Self {
    ROSKernelInit {
      peripheral_base: 0x0,
      memory_regions: [ROSMemoryRegion::new(); MAX_MEM_REGIONS],
    }
  }
}

/// @fn kernel_stub(blob: usize, peripheral_base: usize) -> !
/// @brief   AArch64 kernel stub.
/// @param[in] blob            ATAG or Device Tree blob.
/// @param[in] peripheral_base The peripheral base address.
/// @returns Does not return
/// cbindgen:ignore
#[cfg(target_arch = "aarch64")]
#[no_mangle]
pub extern "C" fn kernel_stub(blob: usize, peripheral_base: usize) -> ! {
  let mut init = ROSKernelInit::new();
  init.peripheral_base = peripheral_base;
  ros_kernel(init)
}

/// @fn kernel_stub(zero: usize, machine_id: usize, blob: usize, peripheral_base: usize) -> !
/// @brief   ARMv7 kernel stub.
/// @param[in] machine_id      The machine ID provided by the bootloader.
/// @param[in] blob            ATAG or Device Tree blob.
/// @param[in] peripheral_base The peripheral base address.
/// @returns Does not return
/// cbindgen:ignore
#[cfg(target_arch = "arm")]
#[no_mangle]
pub extern "C" fn kernel_stub(machine_id: usize, blob: usize, peripheral_base: usize) -> ! {
  let mut init = ROSKernelInit::new();
  init.peripheral_base = peripheral_base;
  ros_kernel(init)
}

/// @fn trap_exception(esr_el1: usize, far_el1: usize)
/// @brief Handles an exception trap.
/// @param[in] esr_el1 Exception Syndrome Register.
/// @param[in] far_el1 Fault Address Register.
/// cbindgen:ignore
#[no_mangle]
pub extern "C" fn trap_exception(esr_el1: usize, far_el1: usize) {}

/// @fn ros_kernel(init: *const ROSKernelInit) -> !
/// @brief   Rust kernel entry point.
/// @param[in] init Pointer to the architecture-dependent setup.
/// @returns Does not return.
fn ros_kernel(init: ROSKernelInit) -> ! {
  peripherals::base::set_peripheral_base_addr(init.peripheral_base);
  peripherals::mini_uart::uart_init();
  startup_messages();
  loop {}
}

/// @fn startup_messages()
/// @brief Print some boring information on startup.
fn startup_messages() {
  let pbase = peripherals::base::get_peripheral_register_addr(0) as usize;
  dbg_print!("=== ROS ===\n");
  dbg_print!("Peripheral Base Address: {:#x}\n", pbase);
}

/// @fn panic(_info: &PanicInfo) -> !
/// @brief   Panic handler.
/// @param[in] info The panic info.
/// @returns Does not return.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
  loop {}
}
