ENTRY(asm_boot)

SECTIONS {
    /* The boot program starts at 0x7c00, and runs for the next 512 bytes, until 0x7e00 */
    .boot = 0x7c00;
    
    /* The "magic number" that marks this as a BIOS boot program */
    . = 0x7c00 + 510;
    .magic_number :
    {
        SHORT(0xaa55)
    }
}
