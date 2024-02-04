use std::{path::Path, process::Command};

const CRATE_ROOT: &str = env!("CARGO_MANIFEST_DIR");

fn main() -> Result<(), String> {
	println!("Launching in QEMU...");
	let root = Path::new(CRATE_ROOT).parent().unwrap();

	let mut qemu = Command::new("qemu-system-x86_64");

	#[cfg(feature = "gdb")]
	qemu.arg("-S").arg("-s");
	qemu.arg("-drive").arg(format!(
		"format=raw,file={},index=0",
		root.join("target").join("bs.bin").display()
	));

	#[cfg(feature = "gdb")]
	println!("Run `target remote localhost:1234` in GDB to connect.");

	let status = qemu.status();

	if status.is_ok_and(|status| status.success()) {
		Ok(())
	} else {
		Err("QEMU failed to run, exiting...".to_string())
	}
}
