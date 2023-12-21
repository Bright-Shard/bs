use std::{
    env,
    fs::{read, File},
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};

/// An easier way to declare a sub-crate in BS, for it to get compiled correctly.
struct Package {
    /// The name of the sub-crate
    name: &'static str,
    /// If the sub-crate's binary needs to be converted from an ELF to raw binary
    needs_transpile: bool,
    /// The target this package needs to be compiled for
    target: &'static str,
}
/// All of the sub-crates in BS
const PACKAGES: [Package; 3] = [
    Package {
        name: "bootstrapper",
        needs_transpile: true,
        target: "target.json",
    },
    Package {
        name: "bootloader",
        needs_transpile: true,
        target: "target.json",
    },
    Package {
        name: "kernel",
        needs_transpile: false,
        target: "x86_64-unknown-none",
    },
];

fn main() {
    let root = env::var("CARGO_MANIFEST_DIR").unwrap();
    let root = Path::new(&root);
    let target = root.join("target");
    let profile = env::var("PROFILE").unwrap();
    let release_mode = profile == "release";
    let llvm_tools = find_llvm_tools();
    let objcopy = llvm_tools.join("llvm-objcopy");

    // Compile all the packages
    PACKAGES.iter().for_each(|pkg| {
        let path = root.join(pkg.name);
        println!("cargo:rerun-if-changed={}", path.display());

        compile(&path, release_mode, pkg.target);
        if pkg.needs_transpile {
            transpile(&path, &objcopy, &profile)
        }
    });

    // Generate the disk image
    let mut disk = File::create(target.join("os.bin")).expect("Failed to create the OS disk image");

    for file in ["bootstrapper", "bootloader"] {
        disk.write_all(
            &read(target.join(format!("{file}.bin")))
                .unwrap_or_else(|e| panic!("Failed to read the OS' {file}: {e}")),
        )
        .unwrap_or_else(|e| panic!("Failed to write {file} to the final OS disk: {e}"))
    }
    disk.write_all(
        &read(
            root.join("kernel")
                .join("target")
                .join("x86_64-unknown-none")
                .join("debug")
                .join("kernel"),
        )
        .unwrap(),
    )
    .unwrap();
}

/// Compiles a BS package
fn compile(package: &Path, release_mode: bool, target: &str) {
    let package_name = package.file_name().unwrap().to_str().unwrap();

    let mut compiler = Command::new("cargo");
    compiler
        .current_dir(package)
        .arg("build")
        .arg("--target-dir")
        .arg("target")
        .arg("--target")
        .arg(target)
        .arg("-Z")
        .arg("build-std=core")
        .arg("-Z")
        .arg("build-std-features=compiler-builtins-mem");

    if release_mode {
        compiler.arg("--release");
    }

    let status = compiler.status();
    if status.is_err() || !status.unwrap().success() {
        panic!("Failed to compile {package_name}");
    }
}

/// Converts a compiled ELF to raw binary that can be put on a disk
fn transpile(package: &Path, objcopy: &Path, profile: &str) {
    let package_name = package.file_name().unwrap().to_str().unwrap();
    let root = package.parent().unwrap();

    let status = Command::new(objcopy)
        .arg("-I")
        .arg("elf64-x86-64")
        .arg("-O")
        .arg("binary")
        .arg("--binary-architecture=i386:x86-64")
        .arg(
            package
                .join("target")
                .join("target")
                .join(profile)
                .join(package_name)
                .to_str()
                .unwrap(),
        )
        .arg(root.join("target").join(format!("{package_name}.bin")))
        .status();

    if status.is_err() || !status.unwrap().success() {
        panic!("Failed to transpile {package_name}");
    }
}

/// Due to the custom target, the binaries in this repo will get compiled into ELFs, but we really need
/// raw binaries to load from the disk. BS uses llvm-objcopy to convert the ELFs to raw binary, and thus
/// needs to find the llvm toolchain.
///
/// This strategy - both using llvm-objcopy and the method for finding it - are taken from phil-opp's work.
/// You can find their version of this code at https://github.com/phil-opp/llvm-tools.
fn find_llvm_tools() -> PathBuf {
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

    llvm_tools.expect(
        "Couldn't find LLVM tools. Make sure the toolchain component `llvm-tools` is installed via rustup."
    )
}
