/* FLASH ORIGIN = 0x00000000 for direct simulator ELF execution.
   Update to 0x00002000 when flashing over the physical XIAO UF2 bootloader. */
MEMORY
{
  FLASH : ORIGIN = 0x00000000, LENGTH = 256K
  RAM   : ORIGIN = 0x20000000, LENGTH = 32K
}