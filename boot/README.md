# BS Bootsector

This contains crates for a BIOS bootloader that loads BS. Booting from BIOS has a *lot* of limitations and issues to work through, so the boot logic is split across several crates. The boot order looks like this:

bootstrapper -> bootloader -> bootstrapper -> elf-loader -> kernel

When the CPU starts up, it loads the BIOS. The BIOS loads the first 512 bytes from the disk into memory, then calls that program. The first 512 bytes in our case is the bootstrapper. The bootstrapper then loads the bootloader, which sets up entering 64-bit mode. The bootloader enters 64-bit mode and returns to the bootstrapper, which then loads the ELF loader. The ELF loader then loads the kernel.

**Note**: BS' bootsector is incomplete. Currently only the bootloader is loaded. I need to add a PCI IDE controller, then have the ELF loader actually load an ELF, before the bootsector is complete.

# Resources
- [This open-source bootloader](https://github.com/X-x-X-x-X-x-X-x-X-x-X-x-X-x-X-x-X/bootloader)
- [Limine bootloader](https://github.com/limine-bootloader/limine)
- [phil-opp's bootloader](https://github.com/rust-osdev/bootloader/tree/main/bios)

There are others that could be potentially useful - like GRUB - but the code is not particularly readable.
