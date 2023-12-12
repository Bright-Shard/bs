# Bootstrapper

This is a tiny (<512 bytes) program that gets loaded by BIOS. 512 bytes is *incredibly* small for any language,
but it's especially small for Rust, which means trying to write a whole bootloader in that little space gets
pretty messy and leads to unreadable code.

Instead of fitting the entire bootloader in that tiny program, the bootloader is placed just outside those 512
bytes. This bootstrapper gets put there instead. It's sole purpose is to figure out how large the bootloader is
and load it into memory - it then immediately calls the bootloader's main function, and lets the bootloader
handle everything from there.

To do this, the bootstrapper uses the BIOS 0x13 syscall, which is used for interacting with disks; specifically,
it uses 0x13's second function, which reads 512-byte sectors from a disk into memory. Starting at sector 2 (the
disk's first sector is the bootstrapper itself) and memory address `0x8000`, the bootstrapper continuously loads
sectors from the disk into memory. At the very end of the bootloader's linker script, aligned with the 512-byte
boundary, are the bytes `0xdeadbeef`, which indicate the end of the bootloader program. Thus, the bootstrapper
keeps loading sectors until it find the bytes `0xdeadbeef` at the end of the sector it just loaded, and then
calls the function at `0x8000` (the bootloader's `main` function).

# Sources

- [This lecture on OS dev](https://www.cs.bham.ac.uk/~exr/lectures/opsys/10_11/lectures/os-dev.pdf) (specifically,
section 3.6, "Reading the Disk")
- [Part 3 of Alex Parker's great OS dev blog posts](http://3zanders.co.uk/2017/10/18/writing-a-bootloader3/)
- Tons of online examples for figuring out how Rust's confusing
[inline assembly](https://doc.rust-lang.org/nightly/reference/inline-assembly.html) works
- The [OSDev Wiki page](https://wiki.osdev.org/Disk_access_using_the_BIOS_(INT_13h)) and the
[Wikipedia page](https://en.wikipedia.org/wiki/INT_13H) for "Int 13h" (the 0x13 syscall)