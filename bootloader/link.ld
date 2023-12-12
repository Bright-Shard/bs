PAGE_TABLE_ENTRY = 3 << 62;
ENTRY(main)

SECTIONS {
    /* The bootstrapper ends at 0x7e00. This gives a bit of padding and gets loaded at 0x8000. */
    . = 0x8000;
    
    /* Actual code */
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

    /* The GDT and page tables are linked to specific locations so that they have known addresses */
    .gdt :
    {
        *(.gdt .gdt.*)
    }
    .gdt.address = ADDR(.gdt);
    
    . = ALIGN(0x1000);
    .page_map_level_4 :
    {
        QUAD(PAGE_TABLE_ENTRY | ADDR(.page_directory_pointer_table))
    }
    . = ALIGN(0x1000);
    .page_directory_pointer_table :
    {
        QUAD(PAGE_TABLE_ENTRY | ADDR(.page_directory))
    }
    . = ALIGN(0x1000);
    .page_directory :
    {
        QUAD(PAGE_TABLE_ENTRY | ADDR(.page_table))
    }
    . = ALIGN(0x1000);
    .page_table :
    {
        .page_table = .;
    }
    
    . = ALIGN(512) - 4;
    .end :
    {
        LONG(0xdeadbeef)
    }
}
