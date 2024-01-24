# BS Bootsector

This contains the crates that boot BS. The bootloader is split into multiple stages, because BIOS only
loads the first 512 bytes of the boot program into memory. 512 bytes is too small for a bootloader, so
those bytes just contain the bootstrapper, which loads the actual bootloader into memory. See the
bootstrapper's docs for more info.

# Resources
- [This open-source bootloader](https://github.com/asjhbdahjgdyilfif/bootloader)
- [Limine bootloader](https://github.com/limine-bootloader/limine)

There are others that could be potentially useful - like GRUB - but the code is not particularly readable.
