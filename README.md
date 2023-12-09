# BS

Welcome to the... uh... BrightSystem?

# Project Status

TLDR: Mid-rewrite and partial disaster! Fun!

`_old` has a functional OS using many great libraries from phil-opp (the guy who made [this](https://os.phil-opp.com/)). You
can run `cargo r` there and see it (requires QEMU to be installed). The code is pretty self-contained because it relies
heavily on dependencies, so you can just go check it out yourself. It was written for a fantastic OS development forum at my
school.

Everything else is my WIP dependency-less rewrite. It currently just builds a BIOS bootloader that prints the character `a`.
I'm working on getting it to load a kernel next, and then will port everything from `_old` over to this new one (and then
will delete `_old`).

# Project Layout

The root crate is empty, and just contains a build script to create the OS binary. Unfortunately in Rust, build scripts can
only run code *before* code is compiled, not after. However, there are tasks that need to be run after code is compiled;
the bootloader, for example, has to be converted from an ELF binary to pure binary that isn't in an executable format. Thus,
the root crate's `build.rs` goes through every module in BS, compiles it, performs any necessary post-compilation tasks,
and then leaves the final binary in `target`.

The bootloader crate contains the BS bootloader. It's currently just written with inline assembly and a linker script that
starts the code at the right address and adds the BIOS magic word that marks it as a BIOS bootloader. As stated above, the
crate will get compiled into an ELF executable, which needs to be converted into raw binary before it's run. The root crate's
build script accomplishes this with `llvm-objcopy`.

The kernel crate is currently empty, but will contain all of the code that's currently in `_old`, ported to the new bootloader.

# Running BS

For the old, more functional version with dependencies, go into `_old` and use a standard `cargo r`/`cargo run`.

For the new version, build the project like normal (`cargo b`/`cargo build`), then run the `bootloader.bin` file in `target`.
To do this in QEMU, for example, run: `cargo b; qemu-system-x86_64 -drive format=raw,file=target/bootloader.bin,index=0`. Again,
the new version is just a WIP BIOS bootloader, so you'll only see the letter `a` printed when running it.
