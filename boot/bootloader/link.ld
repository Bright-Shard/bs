PAGE_TABLE_ENTRY = 3 << 62;
ENTRY(main)

SECTIONS {
    /* The bootstrapper loads the bootloader into memory here. */
    . = 0x7E00;
    
    /* The main function is linked at the very start, so the bootstrapper can just call the address it loaded the bootloader at. */
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
        last bytes. The bootstrapper loads 512 bytes at a time, and looks for 0xdeadbeef to know that
        it's finished loading the bootloader.
    */
    . = ALIGN(512) - 4;
    .end :
    {
        LONG(0xdeadbeef)
    }
}
