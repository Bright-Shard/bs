use std::env;

fn main() {
	// Make rust compile the binary with our link script
	let root = env::var("CARGO_MANIFEST_DIR").unwrap();
	let root = std::path::Path::new(&root);

	println!(
		"cargo:rustc-link-arg-bins=--script={}",
		root.parent().unwrap().join("boot-program.ld").display()
	);
}
