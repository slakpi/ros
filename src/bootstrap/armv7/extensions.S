//! ARMv7a Architecture Extesions

#define ID_MMFR0_VMSA_MASK 0xf
#define VMSAv7_WITH_LONG_DESCRIPTORS 0b0101

/*----------------------------------------------------------------------------*/
/// Check for long page table descriptor support.
///
/// # Returns
///
/// 0 if the CPU supports long table table descriptors, non-zero otherwise.
.global ext_has_long_descriptor_support
ext_has_long_descriptor_support:
// no fn_entry required.

  mrc     p15, 0, r0, c0, c1, 4
  and     r0, r0, #ID_MMFR0_VMSA_MASK
  sub     r0, r0, #VMSAv7_WITH_LONG_DESCRIPTORS
  
// no fn_exit required.
  mov     pc, lr
