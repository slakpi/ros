__page_size = (4 * 1024);

ENTRY(_start)

SECTIONS
{
  .text :
  {
    __kernel_start = .;
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

  /* Reserve 1MiB for the kernel stack. Note the reversal of start and end. */
  .data.stack :
  {
    __kernel_stack_end = .;
    . += (1024 * 1024);
    __kernel_stack_start = .;
  }

  /* Reserve three pages for the kernel's bootstrap page table. */
  .data.pgd :
  {
    __kernel_pages_start = .;
    . += (3 * __page_size);
    __kernel_pages_end = .;
  }

  __kernel_pages_size = __kernel_pages_end - __kernel_pages_start;
  __kernel_size = __kernel_pages_end - __kernel_start;
}
