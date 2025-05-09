include(raspberrypi)

#-------------------------------------------------------------------------------
# Validate the target platform.
#-------------------------------------------------------------------------------
function(validate_platform)
  if(NOT CMAKE_SYSTEM_PROCESSOR MATCHES "(aarch64|armv7)")
    message(FATAL_ERROR "Invalid target platform.")
  endif()

  if(DEFINED RPI_VERSION)
    rpi_validate_platform()
  endif()
endfunction()

#-------------------------------------------------------------------------------
# Validate the build configuration.
#-------------------------------------------------------------------------------
function(validate_build_type)
  if(NOT DEFINED KERNEL_BUILD_TYPE)
    message(WARNING "Defaulting to QEMU build.")
    set(KERNEL_BUILD_TYPE qemu PARENT_SCOPE)
  elseif(NOT KERNEL_BUILD_TYPE MATCHES "(qemu|hardware)")
    message(FATAL_ERROR "Invalid build type.")
  endif()
endfunction()

#-------------------------------------------------------------------------------
# Add architecture-agnostic options for the assembler to the specified target.
#-------------------------------------------------------------------------------
function(target_arch_agnostic_asm_options target)
  set(opts "")

  if(CMAKE_ASM_COMPILER_ID STREQUAL "GNU")
    # -nostdlib: Do not link the standard library.
    # -nostartfiles: Do not use the standard library startup files.
    # -z noexecstack: Prevents executing code in a stack.
    list(APPEND opts -nostdlib -nostartfiles -z noexecstack)
  endif()

  target_link_options(${target} PRIVATE ${opts})
endfunction()

#-------------------------------------------------------------------------------
# Add architecture-dependent options for the assembler to the specified target.
#-------------------------------------------------------------------------------
function(target_arch_asm_options target)
  get_cpu_model(cpu)

  set(opts "")

  if(CMAKE_ASM_COMPILER_ID STREQUAL "GNU")
    if(DEFINED cpu)
      list(APPEND opts -mcpu=${cpu})
    endif()

    # The GNU ARM toolchain only supports software floating-point and does not
    # enable SIMD by default. The GNU AArch64 toolchain does, however, use
    # hardware floating-point and SIMD by default.
    if(CMAKE_SYSTEM_PROCESSOR STREQUAL "aarch64")
      list(APPEND opts -march=armv8-a+nofp+nosimd)
    endif()

    # Turn on position-independent code.
    list(APPEND opts -fPIC)
  endif()

  target_compile_options(${target} PRIVATE ${opts})
endfunction()

#-------------------------------------------------------------------------------
# Build a list of Rust architecture options. Rust's default relocation model is
# position independent.
#-------------------------------------------------------------------------------
function(rust_arch_options opts)
  get_cpu_model(cpu)

  set(tmp ${${opts}})

  if(DEFINED cpu)
    list(APPEND tmp -C target-cpu=${cpu})
  endif()

  # The Rust toolchain only supports software floating-point, but will enable
  # SIMD by default.
  if(CMAKE_SYSTEM_PROCESSOR STREQUAL "aarch64")
    list(APPEND tmp -C target-feature=-neon)
  endif()

  # Quiet Rust's dead code and unused variable warnings in Debug.
  list(APPEND tmp $<$<CONFIG:Debug>:-A dead_code -A unused_variables>)

  set(${opts} ${tmp} PARENT_SCOPE)
endfunction()

#-------------------------------------------------------------------------------
# Get the Rust toolchain for the platform.
#-------------------------------------------------------------------------------
function(rust_arch_target arch)
  if(CMAKE_SYSTEM_PROCESSOR STREQUAL "aarch64")
    set(${arch} aarch64-unknown-none-softfloat PARENT_SCOPE)
  elseif(CMAKE_SYSTEM_PROCESSOR STREQUAL "armv7")
    set(${arch} armv7a-none-eabi PARENT_SCOPE)
  endif()
endfunction()

#-------------------------------------------------------------------------------
# Get the CPU model for the platform, if available.
#-------------------------------------------------------------------------------
function(get_cpu_model cpu)
  if(DEFINED RPI_VERSION)
    rpi_get_cpu_model(tmp)
  endif()

  set(${cpu} ${tmp} PARENT_SCOPE)
endfunction()

#-------------------------------------------------------------------------------
# Get the kernel image file name for the platform.
#-------------------------------------------------------------------------------
function(get_kernel_image_file file)
  if(DEFINED RPI_VERSION)
    rpi_get_kernel_image_file(tmp)
  else()
    message(WARNING "Defaulting to `kernel.img`.")
    set(tmp kernel.img PARENT_SCOPE)
  endif()

  set(${file} ${tmp} PARENT_SCOPE)
endfunction()

#-------------------------------------------------------------------------------
# Get the kernel base address for the platform.
#-------------------------------------------------------------------------------
function(get_kernel_base_address addr)
  if(DEFINED RPI_VERSION)
    rpi_get_kernel_base_address(tmp)
  else()
    message(WARNING "Defaulting to kernel base address of 0x0.")
    set(tmp 0x0)
  endif()

  set(${addr} ${tmp} PARENT_SCOPE)
endfunction()

#-------------------------------------------------------------------------------
# Get the kernel virtual base address for the platform. If KERNEL_VMSPLIT is not
# defined, a 2/2 split is the default. No error checking is done here to prevent
# specifying a split that the target CPU does not support.
#-------------------------------------------------------------------------------
function(get_kernel_virtual_base_address addr split)
  if(DEFINED RPI_VERSION)
    rpi_get_kernel_virtual_base_address(tmp_addr tmp_split)
  else()
    if(CMAKE_SYSTEM_PROCESSOR STREQUAL "aarch64")
      set(tmp_addr 0xffff000000000000)
    elseif(CMAKE_SYSTEM_PROCESSOR STREQUAL "armv7")
      if(KERNEL_VMSPLIT EQUAL 3)
        set(tmp_addr 0xc0000000)
      elseif(KERNEL_VMSPLIT EQUAL 2 OR NOT DEFINED KERNEL_VMSPLIT)
        set(tmp_addr 0x80000000)
      else()
        message(FATAL_ERROR "Invalid virtual memory split.")
      endif()
    endif()

    set(tmp_split ${KERNEL_VMSPLIT})
  endif()

  set(${addr} ${tmp_addr} PARENT_SCOPE)
  set(${split} ${tmp_split} PARENT_SCOPE)
endfunction()

#-------------------------------------------------------------------------------
# Get the kernel page size. Defaults to 4 KiB.
#-------------------------------------------------------------------------------
function(get_kernel_page_size kib)
  if(DEFINED KERNEL_PAGE_SIZE)
    if(CMAKE_SYSTEM_PROCESSOR STREQUAL "aarch64")
      if((NOT KERNEL_PAGE_SIZE EQUAL 4) AND (NOT KERNEL_PAGE_SIZE EQUAL 16) AND
         (NOT KERNEL_PAGE_SIZE EQUAL 64))
         message(FATAL_ERROR "Invalid page size for AArch64")
      else()
        set(${kib} ${KERNEL_PAGE_SIZE} PARENT_SCOPE)
      endif()
    elseif(CMAKE_SYSTEM_PROCESSOR STREQUAL "armv7")
      if((NOT KERNEL_PAGE_SIZE EQUAL 4) AND (NOT KERNEL_PAGE_SIZE EQUAL 64))
        message(FATAL_ERROR "Invalid page size for ARMv7a")
      else()
        set(${kib} ${KERNEL_PAGE_SIZE} PARENT_SCOPE)
      endif()
    endif()
  else()
    set(${kib} 4 PARENT_SCOPE)
  endif()
endfunction()

#-------------------------------------------------------------------------------
# Get the kernel stack page count. Defaults to 2 pages.
#-------------------------------------------------------------------------------
function(get_kernel_stack_page_count pages)
  if (DEFINED KERNEL_STACK_PAGES)
    set(${pages} ${KERNEL_STACK_PAGES} PARENT_SCOPE)
  else()
    set(${pages} 2 PARENT_SCOPE)
  endif()
endfunction()
