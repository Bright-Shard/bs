use std::{
    env,
    path::{Path, PathBuf},
    process::Command,
};

fn main() {
    // Locations of packages in BS
    let root = env::var("CARGO_MANIFEST_DIR").unwrap();
    let root = Path::new(&root);
    let bootloader = root.join("bootloader");

    let profile = env::var("PROFILE").unwrap();

    // LLVM tools (see the docs on find_llvm_tools)
    let llvm_tools = find_llvm_tools().expect(
        "Couldn't find LLVM tools. Make sure the toolchain component `llvm-tools-preview` is installed via rustup."
    );
    let objcopy = llvm_tools.join("llvm-objcopy");

    // Compile the bootloader
    let bootloader_compile = Command::new("cargo")
        .current_dir(bootloader.to_str().unwrap())
        .arg("build")
        .arg("--target-dir")
        .arg("target")
        .status();
    if bootloader_compile.is_err() || !bootloader_compile.unwrap().success() {
        panic!("Failed to compile bootloader");
    }

    // Convert the bootloader to raw binary
    let bootloader_transpile = Command::new(objcopy)
        .arg("-I")
        .arg("elf64-x86-64")
        .arg("-O")
        .arg("binary")
        .arg("--binary-architecture=i386:x86-64")
        .arg(
            bootloader
                .join("target")
                .join("target")
                .join(profile)
                .join("bootloader")
                .to_str()
                .unwrap(),
        )
        .arg(root.join("target").join("bootloader.bin").to_str().unwrap())
        .status();

    if bootloader_transpile.is_err() || !bootloader_transpile.unwrap().success() {
        panic!("Failed to convert the bootloader to binary");
    }
}

/// Due to the custom target, the binaries in this repo will get compiled into ELFs, but we really need
/// raw binaries to load from the disk. BS uses llvm-objcopy to convert the ELFs to raw binary, and thus
/// needs to find the llvm toolchain.
///
/// This strategy - both using llvm-objcopy and the method for finding it - are taken from phil-opp's work
/// in Rust OS development. You can find their version of this code at https://github.com/phil-opp/llvm-tools.
fn find_llvm_tools() -> Option<PathBuf> {
    println!("Finding LLVM");
    let mut llvm_tools = None;
    let sysroot = Command::new("rustc")
        .arg("--print")
        .arg("sysroot")
        .output()
        .unwrap();
    let toolchain_root = PathBuf::from(std::str::from_utf8(&sysroot.stdout).unwrap().trim());
    let toolchain_root = toolchain_root.join("lib").join("rustlib");
    for lib in toolchain_root.read_dir().unwrap() {
        let bin_dir = lib.unwrap().path().join("bin");
        if bin_dir.join("llvm-objcopy").exists() {
            llvm_tools = Some(bin_dir);
        }
    }

    llvm_tools
}
