use std::{path::Path, process::Command};

const CRATE_ROOT: &str = env!("CARGO_MANIFEST_DIR");

fn main() -> Result<(), String> {
	println!("Launching in QEMU...");
	let root = Path::new(CRATE_ROOT).parent().unwrap();

	let qemu = Command::new("qemu-system-x86_64")
		.arg("-drive")
		.arg(format!(
			"format=raw,file={},index=0",
			root.join("target").join("bs.bin").display()
		))
		.status();

	if qemu.is_ok_and(|qemu| qemu.success()) {
		Ok(())
	} else {
		Err("QEMU failed to run, exiting...".to_string())
	}
}
