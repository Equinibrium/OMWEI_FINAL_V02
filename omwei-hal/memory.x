/* OMWEI Equinibrium SoC Memory Map */

/* Stack allocation for 4 Harts (128KB each) */
STACK_SIZE = 128K;

/* FLASH (Peripheral Port) - 512MB */
MEMORY
{
  FLASH (rx) : ORIGIN = 0x20000000, LENGTH = 512M
  RAM (rwx) : ORIGIN = 0x80000000, LENGTH = 512M
  ITIM (rx) : ORIGIN = 0x01800000, LENGTH = 16K
  DLS (rwx) : ORIGIN = 0x18000000, LENGTH = 64K
}
