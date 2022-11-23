/**
 * This file is for 32-bit QEMU builds only. QEMU loads the kernel at 0x10000
 * rather than 0x8000.
 */

ENTRY(_start)
 
SECTIONS
{
  . = 0x10000;
  __start = .;
  __text_start = .;
  .text :
  {
      KEEP(*(.text.boot))
      *(.text)
  }
  . = ALIGN(4096); /* align to page size */
  __text_end = .;

  __rodata_start = .;
  .rodata :
  {
      *(.rodata)
  }
  . = ALIGN(4096); /* align to page size */
  __rodata_end = .;

  __data_start = .;
  .data :
  {
      *(.data)
  }
  . = ALIGN(4096); /* align to page size */
  __data_end = .;

  __bss_start = .;
  .bss :
  {
      bss = .;
      *(.bss)
  }
  . = ALIGN(4096); /* align to page size */
  __bss_end = .;
  __bss_size = __bss_end - __bss_start;
  __end = .;
}
