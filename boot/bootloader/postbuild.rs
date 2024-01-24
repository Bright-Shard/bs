```cargo
[dependencies.build-tools]
path = "../../lib/build-tools"
```

fn main() {
    // Cargo outputs an ELF; we want raw binary to put on the disk.
    build_tools::elf2bin(Some("boot-target"), "bootloader");
}
