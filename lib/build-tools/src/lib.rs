use std::{env, fs, path::PathBuf, process::Command};

/// Rust outputs an ELF file for custom targets, but we need raw binary.
/// This uses llvm-objcopy to convert the ELF to binary.
pub fn elf2bin(custom_target: Option<&str>, binary: &str) {
	let root = env::var("BARGO_ROOT").unwrap();
	let profile = env::var("PROFILE").unwrap();

	let mut input = PathBuf::from(&root);
	input.push("target");
	if let Some(custom_target) = custom_target {
		input.push(custom_target);
	}
	input.push(profile);
	input.push(binary);

	let mut output = PathBuf::from(root);
	output.push("target");
	output.push("bs-bins");
	if !output.exists() {
		fs::create_dir(&output).unwrap();
	}
	output.push(format!("{binary}.bin"));

	let cmd = Command::new(get_llvm_objcopy())
		.arg("-I")
		.arg("elf64-x86-64")
		.arg("-O")
		.arg("binary")
		.arg("--binary-architecture=i386:x86-64")
		.arg(input.to_str().unwrap())
		.arg(output.to_str().unwrap())
		.status();

	if cmd.is_err() || !cmd.unwrap().success() {
		panic!("Failed to convert `{binary}` into raw binary")
	}
}

/// Finds the `llvm-objcopy` binary, which is installed with the `llvm-tools` toolchain component.
/// This is unapologetically stolen from phil-opp's crate: https://github.com/phil-opp/llvm-tools
pub fn get_llvm_objcopy() -> PathBuf {
	let mut llvm_tools = None;
	let sysroot = Command::new("rustc")
		.arg("--print")
		.arg("sysroot")
		.output()
		.unwrap();
	let toolchain_root = PathBuf::from(std::str::from_utf8(&sysroot.stdout).unwrap().trim());
	let toolchain_root = toolchain_root.join("lib").join("rustlib");
	for lib in toolchain_root.read_dir().unwrap() {
		let objcopy = lib.unwrap().path().join("bin").join("llvm-objcopy");
		if objcopy.exists() {
			llvm_tools = Some(objcopy);
		}
	}

	llvm_tools.expect(
        "Couldn't find LLVM tools. Make sure the toolchain component `llvm-tools` is installed via rustup."
    )
}
