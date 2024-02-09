# Bootloader

This is the core bootloader for BS. It takes care of entering 64-bit mode, then hands things off to the elf-loader for it to load the kernel.

When the CPU turns on, it starts in "real mode", a limited 16-bit environment. The CPU only has access to about 1mb of memory since it's working with 16 bits at a time - and only about half of that is actually useable, as the rest is reserved for BIOS, memory-mapped IO, etc. This is a pretty limiting size for the kernel, so this bootloader enters 64-bit, giving the 64-bit ELF loader access to all the computer's memory.

After entering 64-bit mode, the bootloader returns to the bootstrapper, which will then load and call the ELF loader.

# Building

Build with Bargo: `bargo build -p bootloader`

In the `target` folder, there will now be a `bs-bins` folder. Inside there will be a `bootloader.bin` file that contains the raw bootloader binary.

# Sources

- [phil-opp's bootloader crate](https://github.com/rust-osdev/bootloader/blob/main/bios): This one is also written in Rust and is accomplishing a similar goal, so it's a pretty good example to look at.
- [This great blog post by Alan Foster](https://www.alanfoster.me/posts/writing-a-bootloader/): The blog unfortunately doesn't contain a ton of info itself, but it links to other great resources and clarifies some x86 concepts.
- [Another great blog post, by Alex Parker](http://3zanders.co.uk/2017/10/13/writing-a-bootloader/): This one's in 3 parts and has a *lot* of good information.
- [This lecture on OS dev](https://www.cs.bham.ac.uk/~exr/lectures/opsys/10_11/lectures/os-dev.pdf): I'm not convinced this is actually a lecture, since it seems to have lots of typos and broken links, but it walks step by step through many useful concepts.
- [The LD docs for the linker script](https://sourceware.org/binutils/docs/ld/Scripts.html): The the only reason I was able to make the linker script for this.
- [The OSDev Wiki](https://wiki.osdev.org): This is just a great resource all around.
