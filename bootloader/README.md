# Bootloader

This is the BIOS bootloader for BS. It's BIOS instead of UEFI because, as far as I know, base QEMU only loads BIOS
images, and right now QEMU is my main target for running BS.

# Sources

Here's a bunch of the sources I used to figure out how to do this:

- [phil-opp's bootloader crate](https://github.com/rust-osdev/bootloader/blob/main/bios)
- [This great blog post by Alan Foster](https://www.alanfoster.me/posts/writing-a-bootloader/)
- [The LD docs for the linker script](https://sourceware.org/binutils/docs/ld/Scripts.html)
