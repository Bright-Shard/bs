/* A link script for any programs loaded after the bootstrapper. */

ENTRY(main)

SECTIONS {
    /* The bootstrapper loads boot programs into memory at 0x7E00. */
    . = 0x7E00;
    
    /*
        We make the very start of the file be the main fn, so the bootstrapper can just call
        0x7E00 as a function to start the boot program.
    */
    .main :
    {
        *(.main)
    }

    /* All the other parts of the boot program. */
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
        The bootstrapper loads 512 bytes off the disk at a time. We align the file to 512 bytes,
        so the bootstrapper loads the whole thing, and mark the end of the file with 0xdeadbeef.
        The bootstrapper stops loading the file once it hits 0xdeadbeef.
    */
    . += 4; /* Make sure there's 4 bytes of room before the next 512-byte alignment. Prevents edge
               cases where, for example, the boot program may already be exactly 512 bytes. */
    . = ALIGN(512) - 4;
    .end :
    {
        LONG(0xdeadbeef)
    }
}
