# BS

Welcome to the... uh... BrightSystem?

It's an x86_64 OS written entirely from scratch (no dependencies), made by yours truly.

_This project was written without the assistance of GitHub CodeStealer, OpenAI's ChatGPThief, or other similar content-stealing predictive algorithms._

# Project Status

TLDR: Mid-rewrite and partial disaster! Fun!

`_old` has a functional kernel that uses the great libraries from [phil-opp](https://os.phil-opp.com/) and a few others. The code isn't super well documented, but it's not _too_ complicated, so it should be pretty readable. It was made for an awesome OS development forum at my highschool.

Everything else is my WIP dependency-less rewrite. Right now, it only has a BIOS bootloader that can enter 64-bit mode. The next step is to implement an ELF parser, then an ELF loader, and then load the kernel (which will simply be an ELF file).

The new version of BS is extremely well documented, including resources for further research. Part of my goal is to make this excellent demo code for future programmers to reference and learn from.

# Project Layout

Every folder has a README and is hopefully self-explanatory, but here's a rough table of contents for this repo:

- `boot`: All the crates in BS' bootloader.
- `kernel`: BS' kernel (currently empty until the ELF loader is written).
- `lib`: Helper libraries used by BS. This has build tools, Frieren (the WIP ELF loader), and a common library (which will soon be split into multiple crates). These crates have their own libraries because they're used by multiple crates in BS (eg, the bootloader loads an ELF, but the final operating system will be able to as well).
- `qemu`: A crate that builds BS into a final disk and launches it in QEMU.

# Building & Running BS

For the old, more functional version of BS with dependencies, run `cargo r` in the `_old` folder.

The new version uses my custom build system, [bargo](https://github.com/bright-shard/bargo), so you'll need to install that first (fear not - bargo only has 1 dependency, a dependency-free toml parser, and should compile in seconds). Bargo is sort of a wrapper around Cargo, so you can use it almost exactly the same way - `bargo b` to build, `bargo r` to run, `-r` for release mode, etc.

If you're wondering why BS uses bargo instead of Cargo, it's because Cargo doesn't have all the features I need. I need post-build scripts, and the ability to use `build-std` for multiple targets, since the bootloader has a different target than the kernel and OS. I tried for hours, but could not come up with a sane way to implement this in vanilla Cargo.

Both versions run in QEMU, so make sure that's installed first. If you want to build and run it manually, the command being used under-the-hood essentially boils down to this: `cargo b; qemu-system-x86_64 -drive format=raw,file=target/bs.bin,index=0`. For the old version, you'll want to load the file `_old/disk.bin` instead of `target/bs.bin`.

# Other OSDev Resources

- https://wiki.osdev.org/
- https://forum.osdev.org/
- http://www.brokenthorn.com/Resources/OSDevIndex.html
- https://github.com/rust-osdev/
- https://gitlab.redox-os.org/redox-os/
- https://www.intel.com/content/www/us/en/developer/articles/technical/intel-sdm.html
- https://www.amd.com/content/dam/amd/en/documents/processor-tech-docs/programmer-references/40332.pdf
- https://read.seas.harvard.edu/cs161 & https://github.com/CS161
