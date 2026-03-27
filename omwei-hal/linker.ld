/* OMWEI Equinibrium SoC Linker Script */

/* Stack allocation for 4 Harts (128KB each) - MOVED TO SAFE AREA */
STACK_SIZE = 128K;

/* FLASH (Peripheral Port) - 512MB */
MEMORY
{
  FLASH (rx) : ORIGIN = 0x20000000, LENGTH = 512M
  RAM (rwx) : ORIGIN = 0x80000000, LENGTH = 512M
  ITIM (rx) : ORIGIN = 0x01800000, LENGTH = 16K
  DLS (rwx) : ORIGIN = 0x18000000, LENGTH = 64K
}

/* Entry point - ensure it starts exactly at 0x80000000 */
ENTRY(_start)

/* Section placement */
SECTIONS
{
  /* Start at RAM base address */
  . = 0x80000000;
  
  /* Text section - _start MUST be at very beginning */
  .text.boot :
  {
    KEEP(*(.text.boot))   /* THIS MUST BE FIRST - our boot code */
  } > RAM
  
  /* Then all other code */
  .text : ALIGN(4)
  {
    *(.text.traps)
    *(.text.*)
    *(.rodata.*)
    *(.sdata.*)
    *(.srodata.*)
  } > RAM

  /* Read-write data */
  .data :
  {
    KEEP(*(.data.*))
    KEEP(*(.sdata.*))
  } > RAM

  /* BSS section */
  .bss (NOLOAD) :
  {
    __bss_start = .;
    KEEP(*(.bss.*))
    KEEP(*(.sbss.*))
    __bss_end = .;
  } > RAM

  /* Stack allocation in SAFE AREA - starts at 0x80080000 */
  .stack (NOLOAD) :
  {
    __hart0_stack_start = .;
    . += STACK_SIZE;
    __hart0_stack_end = .;
    
    __hart1_stack_start = .;
    . += STACK_SIZE;
    __hart1_stack_end = .;
    
    __hart2_stack_start = .;
    . += STACK_SIZE;
    __hart2_stack_end = .;
    
    __hart3_stack_start = .;
    . += STACK_SIZE;
    __hart3_stack_end = .;
  } > RAM

  /* Heap */
  __heap_start = .;
  . = ORIGIN(RAM) + LENGTH(RAM) - .;
  __heap_end = .;
}
