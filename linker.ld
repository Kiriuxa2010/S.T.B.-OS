ENTRY(_start)

SECTIONS {
  . = 1M;
  .text : {
    *(.text)
  }
  .rodata : {
    *(.rodata)
  }
  .data : {
    *(.data)
  }
  .bss : {
    *(.bss)
  }
  /DISCARD/ : {
    *(.comment)
  }
  .multiboot : ALIGN(4) {
    MULTIBOOT_HEADER_MAGIC;
    MULTIBOOT_HEADER_FLAGS;
    MULTIBOOT_CHECKSUM;
    *(.multiboot)
  }
  .multiboot_header : {
    PROVIDE(multiboot_header_start = .);
    *(.multiboot_header)
    PROVIDE(multiboot_header_end = .);
  }
}

