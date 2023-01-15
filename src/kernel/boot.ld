__page_size = 4096;

ENTRY(_start)

SECTIONS
{
  .text :
  {
    __text_start = .;

    KEEP(*(.text.boot))
    *(.text)

    . = ALIGN(__page_size);
    __text_end = .;
  }

  .rodata :
  {
    __rodata_start = .;

    *(.rodata)

    . = ALIGN(__page_size);
    __rodata_end = .;
  }

  .data :
  {
    __data_start = .;

    *(.data)

    . = ALIGN(__page_size);
    __data_end = .;
  }

  .bss :
  {
    __bss_start = .;

    bss = .;
    *(.bss .bss.*)

    . = ALIGN(__page_size);
    __bss_end = .;
  }
  
  __bss_size = __bss_end - __bss_start;
  __kernel_size = __bss_end - __text_start;
}
