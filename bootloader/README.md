# Bootloader

This is the BIOS bootloader for BS. It's BIOS instead of UEFI because, as far as I know, base QEMU only loads BIOS
images, and right now QEMU is my main target for running BS.

# Sources

Here's a bunch of the sources I used to figure out how to do this:

- [phil-opp's bootloader crate](https://github.com/rust-osdev/bootloader/blob/main/bios): This one is also written
in Rust and is accomplishing a similar goal, so it's a pretty good example to look at.
- [This great blog post by Alan Foster](https://www.alanfoster.me/posts/writing-a-bootloader/): The blog unfortunately
doesn't contain a ton of info itself, but it links to other great resources and clarifies some x86 concepts.
- [Another great blog post, by Alex Parker](http://3zanders.co.uk/2017/10/13/writing-a-bootloader/): This one's in 3
parts and has a *lot* of good information.
- [This lecture on OS dev](https://www.cs.bham.ac.uk/~exr/lectures/opsys/10_11/lectures/os-dev.pdf): I'm not convinced
this is actually a lecture, since it seems to have lots of typos and broken links, but it walks step by step through
pretty much every concept I used for my bootloader.
- [The LD docs for the linker script](https://sourceware.org/binutils/docs/ld/Scripts.html): They're the only reason I
was able to make the linker script for this. They're pretty great docs.
