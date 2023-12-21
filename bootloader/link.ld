PAGE_TABLE_ENTRY = 3 << 62;
ENTRY(main)

SECTIONS {
    /* The bootstrapper loads the bootloader into memory here. */
    . = 0x4000;
    
    /* The main function is linked at the very start, so the bootstrapper can just call the function that's at 0x8000. */
    .main :
    {
        *(.main)
    }
    .rust :
    {
        *(.text .text.*)
        *(.rodata .rodata.*)
        /*
            I've also seen .data, .data.*, .got, and .got.* linked here, but my code works
            without them so I don't think they're necessary.
        */
    }
    
    /*
        The bootloader is aligned to 512 bytes, the size of a disk sector, and puts 0xdeadbeef as its
        last bytes so that the bootstrapper knows when it's finished loading the bootloader.
    */
    . = ALIGN(512) - 4;
    .end :
    {
        LONG(0xdeadbeef)
    }
}
