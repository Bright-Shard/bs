# BS

Welcome to the... uh... BrightSystem?

It's an x86_64 OS written entirely from scratch, made by yours truly.

# Project Status

TLDR: Mid-rewrite and partial disaster! Fun!

`_old` has a functional OS using great libraries from [phil-opp](https://os.phil-opp.com/) and a few others. The code
isn't super well documented, but it's not *too* complicated, so it should be pretty readable. It was made for an awesome
OS development forum at my highschool.

Everything else is my WIP dependency-less rewrite. Right now, it only has a BIOS bootloader that loads a GDT and prints
some text to the screen; I'm currently working on adding paging and 64-bit mode to it. After that, the plan is to add an ELF
loader that will load the kernel into memory (the kernel will be compiled into an ELF). The kernel will be nearly copy/paste
from the current one in `_old`.

The new version is extremely well documented, including resources for further research. Part of my goal is to make this great
demo code for future programmers to reference and learn from.

# Project Layout

The **root** crate just contains a build script that correctly compiles all of the sub-crates, and a runner that launches BS in
QEMU.

The **bootstrapper** crate is what gets loaded from BIOS. Unfortunately, BIOS programs are limited to 512 bytes, which is hard
to do in a Rust program. So the bootstrapper is an extremely small program that just loads the bootloader from disk (without
the dumb 512-byte limit) and runs it.

The **bootloader** crate contains the BS bootloader, which is responsible for all of the setup the kernel needs to run correctly.
It has to enable memory paging, 64-bit mode (the processor boots into 16-bit mode, believe it or not), and a few other small things
that paging and 64-bit mode require to be enabled. It then loads the kernel into memory and runs it. (Note: Currently, the
bootloader is WIP and only enables 64-bit mode and prints some text.)

The **kernel** crate is currently empty, but will contain all of the code that's currently in `_old`, ported to the new bootloader.

You can go into each crate's folder and read its README for more information.

# Running BS

For the old, more functional version with dependencies, run `cargo r`/`cargo run` in the `_old` folder.

For the new version, just run `cargo r` like normal.

Both versions run in QEMU, so make sure that's installed first. If you want to build and run it manually, the command being used
under-the-hood essentially boils down to this: `cargo b; qemu-system-x86_64 -drive format=raw,file=target/os.bin,index=0`. For the
old version, you'll want to load the file `_old/disk.bin` instead of `target/os.bin`.
