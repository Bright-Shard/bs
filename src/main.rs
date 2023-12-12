use std::{path::Path, process::Command};

fn main() {
    let root = std::env::var("CARGO_MANIFEST_DIR").expect(
        "This should only be run with `cargo r`, as it just launches QEMU and is not the actual OS."
    );
    let root = Path::new(&root);

    let qemu = Command::new("qemu-system-x86_64")
        .arg("-drive")
        .arg(format!(
            "format=raw,file={},index=0",
            root.join("target").join("os.bin").display()
        ))
        .status();
    if qemu.is_err() || !qemu.unwrap().success() {
        panic!("QEMU failed to run - make sure it's installed.");
    }
}
