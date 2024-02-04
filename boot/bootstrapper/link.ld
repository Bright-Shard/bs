ENTRY(asm_main)

SECTIONS {
    /* The boot program starts at 0x7c00, and runs for the next 512 bytes, until 0x7e00. */
    . = 0x7c00;
    .asm :
    {
        .asm_main
        .asm_code
        *(.asm .asm.*)
    }

    /* Rust code */
    .rust :
    {
        *(.text .text.*)
        *(.rodata .rodata.*)
        /*
            I've also seen .data, .data.*, .got, and .got.* linked here, but my code works
            without them so I don't think they're necessary.
        */
    }

    /* The "magic number" that marks this as a BIOS boot program */
    . = 0x7c00 + 510;
    .magic_number :
    {
        SHORT(0xaa55)
    }
}
