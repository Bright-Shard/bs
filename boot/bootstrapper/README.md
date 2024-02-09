# Bootstrapper

This is the tiny (<512 bytes!) program that BIOS loads when the computer starts. Obviously, 512 bytes is too small to do everything a bootloader needs to do, so this program just loads other boot programs and jumps to those to let them do the heavy lifting. The bootstrapper is responsible for loading all the other boot programs in BS' bootloader.

Currently, the bootstrapper uses BIOS' INT 13h interrupt to read from disks. This reads 512 bytes at a time into memory. Each boot program has `0xDEADBEEF` at the end of its file (aligned with 512 bytes), so the bootstrapper just keeps loading until it finds `0xDEADBEEF`. All boot programs get loaded to `0x7E00`, one after another.

In the future, the bootstrapper will use a PCI IDE controller to read from disk.

# Building

Build with Bargo: `bargo build -p bootstrapper`

In the `target` folder, there will now be a `bs-bins` folder. Inside there will be a `bootstrapper.bin` file
that contains the raw bootstrapper binary.

# Sources

- [This lecture on OS dev](https://www.cs.bham.ac.uk/~exr/lectures/opsys/10_11/lectures/os-dev.pdf) (specifically, section 3.6, "Reading the Disk")
- [Part 3 of Alex Parker's great OS dev blog posts](http://3zanders.co.uk/2017/10/18/writing-a-bootloader3/)
- Tons of online examples for figuring out how Rust's confusing [inline assembly](https://doc.rust-lang.org/nightly/reference/inline-assembly.html) works
- The [OSDev Wiki page](https://wiki.osdev.org/Disk_access_using_the_BIOS_(INT_13h)) and the [Wikipedia page](https://en.wikipedia.org/wiki/INT_13H) for "Int 13h" (the 0x13 BIOS call)
