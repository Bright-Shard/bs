# elf-loader

This is the third (and final stage of BS' boot process). It uses Frieren to load the kernel into memory and start it. It is currently incomplete - I need to add a PCI IDE loader to BS so it can load the ELF loader, then make this actually load an ELF.
