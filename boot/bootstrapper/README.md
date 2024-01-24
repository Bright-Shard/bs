# Bootstrapper

This is a tiny (<512 bytes) program that gets loaded by BIOS. 512 bytes is *incredibly* small for any language,
but it's especially small for Rust, which means trying to write a whole bootloader in that little space gets
pretty messy and leads to unreadable code (if it's even possible).

Instead of fitting the entire bootloader in that tiny program, the bootloader is placed outside those 512 bytes,
and this bootstrapper is placed in those 512 bytes instead. The sole job of the bootstrapper is to load the
bootloader into memory. This allows the bootloader to be virtually any size (theoretically, there's a memory
limit of 512kib, but the bootloader shouldn't be that large anyways). The bootstrapper will find the end of
the bootloader at runtime and load it into memory.

To do this, the bootstrapper uses the BIOS 0x13 syscall, which is used for interacting with disks; specifically,
it uses 0x13's second function, which reads 512-byte sectors from a disk into memory. Starting at sector 2 (the
disk's first sector is the bootstrapper itself) and memory address `0x7E00`, the bootstrapper continuously loads
sectors from the disk into memory. At the very end of the bootloader's linker script, aligned with the 512-byte
boundary, are the bytes `0xdeadbeef`; this allows the bootstrapper to keep loading disk sectors into memory
until it finds the `0xdeadbeef` bytes, then jump to the bootloader's main function.

# Building

Build with Bargo: `bargo build -p bootstrapper`

In the `target` folder, there will now be a `bs-bins` folder. Inside there will be a `bootstrapper.bin` file
that contains the raw bootstrapper binary.

# Sources

- [This lecture on OS dev](https://www.cs.bham.ac.uk/~exr/lectures/opsys/10_11/lectures/os-dev.pdf) (specifically,
section 3.6, "Reading the Disk")
- [Part 3 of Alex Parker's great OS dev blog posts](http://3zanders.co.uk/2017/10/18/writing-a-bootloader3/)
- Tons of online examples for figuring out how Rust's confusing
[inline assembly](https://doc.rust-lang.org/nightly/reference/inline-assembly.html) works
- The [OSDev Wiki page](https://wiki.osdev.org/Disk_access_using_the_BIOS_(INT_13h)) and the
[Wikipedia page](https://en.wikipedia.org/wiki/INT_13H) for "Int 13h" (the 0x13 syscall)