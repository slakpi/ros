#-------------------------------------------------------------------------------
# Validate the Raspberry Pi version specified on the command line for the
# target platform.
#
# AArch64 - Raspberry Pi 2 (rev 1.2), 3 & 4
# ARMv7a  - Raspberry Pi 2, 2 (rev 1.2), 3 & 4
#
# For the Raspberry Pi Zero 2W, use version 3.
#-------------------------------------------------------------------------------
function(rpi_validate_platform)
  if(CMAKE_SYSTEM_PROCESSOR STREQUAL "aarch64")
    if(NOT (RPI_VERSION MATCHES "^(2_2|3|4)$"))
      message(FATAL_ERROR "Invalid Raspberry Pi version for AArch64.")
    endif()
  elseif(CMAKE_SYSTEM_PROCESSOR STREQUAL "armv7")
    if(NOT (RPI_VERSION MATCHES "^(2|2_2|3|4)$"))
      message(FATAL_ERROR "Invalid Raspberry Pi version for ARMv7.")
    endif()
  else()
    message(FATAL_ERROR "Invalid architecture for Raspberry Pi.")
  endif()
endfunction()

#-------------------------------------------------------------------------------
# Get the CPU model for the Raspberry Pi version specified on the command line.
#
#   Raspberry Pi                 CPU Model
#   ---------------------------------------
#   2                            Cortex A7
#   2 (rev 1.2), 3, Zero 2W      Cortex A53
#   4                            Cortex A72
#-------------------------------------------------------------------------------
function(rpi_get_cpu_model cpu)
  if(RPI_VERSION STREQUAL "4")
    set(${cpu} "cortex-a72" PARENT_SCOPE)
  elseif(RPI_VERSION MATCHES "^(2_2|3)$")
    set(${cpu} "cortex-a53" PARENT_SCOPE)
  elseif(RPI_VERSION STREQUAL "2")
    set(${cpu} "cortex-a7" PARENT_SCOPE)
  endif()
endfunction()

#-------------------------------------------------------------------------------
# Get the kernel image file name expected by the Raspberry Pi bootloader for the
# target platform.
#
# ARMv7a:
#
#   Raspberry Pi                 Kernel Name
#   -----------------------------------------
#   2, 2 (rev 1.2), 3, Zero 2W   kernel7.img
#   4                            kernel7l.img
#
# AArch64:
#
#   Raspberry Pi                 Kernel Name
#   -----------------------------------------
#   2 (rev 1.2), 3, 4            kernel8.img
#-------------------------------------------------------------------------------
function(rpi_get_kernel_image_file file)
  if(CMAKE_SYSTEM_PROCESSOR STREQUAL "aarch64")
    set(${file} kernel8.img PARENT_SCOPE)
  elseif(CMAKE_SYSTEM_PROCESSOR STREQUAL "armv7")
    if(RPI_VERSION MATCHES "^(2|2_2|3)$")
      set(${file} kernel7.img PARENT_SCOPE)
    elseif(RPI_VERSION STREQUAL "4")
      set(${file} kernel7l.img PARENT_SCOPE)
    endif()
  endif()
endfunction()

#-------------------------------------------------------------------------------
# Get the kernel base address expected by the Raspberry Pi bootloader. The
# Raspberry Pi bootloader expects the kernel at 0x80000 for AArch64. For ARM,
# the Raspberry Pi bootloader expects the kernel at 0x8000, however QEMU expects
# it at 0x10000.
#-------------------------------------------------------------------------------
function(rpi_get_kernel_base_address addr)
  if(CMAKE_SYSTEM_PROCESSOR STREQUAL "aarch64")
    set(${addr} 0x80000 PARENT_SCOPE)
  elseif(CMAKE_SYSTEM_PROCESSOR STREQUAL "armv7")
    if(${QEMU_BUILD})
      set(${addr} 0x10000 PARENT_SCOPE)
    else()
      set(${addr} 0x8000 PARENT_SCOPE)
    endif()
  endif()
endfunction()

#-------------------------------------------------------------------------------
# Get the kernel virtual base address for the Raspberry Pi version specified on
# the command line.
#
#   NOTE: The ARMv7 bootstrap code uses short page descriptors for the 2:2 split
#         and long page descriptors for the 3:1 split. This is entirely a
#         temporary thing just to test setting up both types of tables.
#-------------------------------------------------------------------------------
function(rpi_get_kernel_virtual_base_address addr split)
  if(CMAKE_SYSTEM_PROCESSOR STREQUAL "aarch64")
    set(${addr} 0xffff000000000000 PARENT_SCOPE)
  elseif(CMAKE_SYSTEM_PROCESSOR STREQUAL "armv7")
    if (DEFINED KERNEL_VMSPLIT)
      # If KERNEL_VMSPLIT specifies anything other than a 3:1 or 2:2 split,
      # we'll catch it below. If the CPU does not support a 3:1 split, we'll
      # catch it at runtime in the bootstrap code and halt.
      set(tmp_split ${KERNEL_VMSPLIT})
    else()
      # The Cortex A7 used by the original Raspberry Pi 2 and the Cortex A53
      # used by the Raspberry Pi's 2 (rev 1.2), 3, and 4 support LPAE, so just
      # default to a 3:1 split. In the case of the 2, 2 (rev 1.2), and 3, this
      # may NOT be the optimal split.
      set(tmp_split 3)
    endif()
    
    if(tmp_split EQUAL 3)
      set(${addr} 0xc0000000 PARENT_SCOPE)
    elseif(tmp_split EQUAL 2)
      set(${addr} 0x80000000 PARENT_SCOPE)
    else()
      message(FATAL_ERROR "Invalid virtual memory split.")
    endif()
  endif()

  set(${split} ${tmp_split} PARENT_SCOPE)
endfunction()
