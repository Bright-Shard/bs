# BS Bootsector

This contains crates that boot BS. Booting is unfortunately quite complicated, so the bootloader is split into multiple crates. BS basically has to deal with these issues:

1. The computer boots in 16-bit mode: Despite being a 64-bit processor, when the computer boots, it's in 16-bit mode. BS has to set several flags to enable 64-bit mode itself.
2. 16-bit mode has limited memory: In 16-bit mode, only the first mib of memory can be accessed.
3. The bootloader has to be 512 bytes: The CPU only loads 512 bytes from disk into memory. This means the bootloader must fit into 512 bytes.

Because the computer starts in 16-bit mode, but we enter 64-bit mode, we have to build code for multiple bitnesses. In Rust, different bitnesses requires different targets, and a Rust crate can only be built for one target. This means we have to have multiple crates to compile for the 16-bit and 64-bit targets.

To solve the limited memory issue, BS just enters 64-bit mode before loading the kernel into memory. The bootloader and bootstrapper are in 16-bit mode, but the ELF loader that lods the kernel is in 64-bit mode, so it has access to all the computer's memory.

Finally, to solve the 512-byte size constraint, BS splits the bootloader into multiple stages. The first stage is the bootstrapper, whose sole responsibility is to load other programs into memory. The next stage is the bootloader, which enters 64-bit mode. The final stage is the elfloader, which loads the kernel into memory and jumps to it. The elfloader is currently not implemented.

Splitting the bootloader into multiple stages creates one last issue: It's not clear where programs get loaded into memory, if they're all loaded after each other. The bootstrapper is loaded at 0x7c00, and is 512 bytes, which means the bootloader will get loaded at 0x7c00 + 512 bytes. However, we don't know the size of the bootloader without hardcoding it (which makes code hard to maintain later), so we don't know where the elfloader will get loaded in memory. To solve this, the bootloader actually jumps back to the bootstrapper and lets the bootstrapper load the elfloader into memory.

# Resources
- [This open-source bootloader](https://github.com/X-x-X-x-X-x-X-x-X-x-X-x-X-x-X-x-X/bootloader)
- [Limine bootloader](https://github.com/limine-bootloader/limine)

There are others that could be potentially useful - like GRUB - but the code is not particularly readable.
