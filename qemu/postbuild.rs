```cargo
package.edition = "2021"
```

//! Builds BS into a bootable disk. This is implemented as a postbuild because postbuilds will always run
//! after a crate has compiled, but normal builds will not be run if a crate isn't recompiled.

use std::{
	env,
	fs::{self, File},
	io::Write,
	path::PathBuf,
};

/// Thanks to Bargo's binary dependencies and post-build scripts, BS is already built. This just has to copy
/// the final binaries into one file that will act like a disk, then load that file in QEMU.
fn main() {
	let target = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
		.parent()
		.unwrap()
		.join("target");
	let profile = env::var("PROFILE").unwrap();
	let bs_bins = target.join("bs-bins");
	let mut output = File::create(target.join("bs.bin")).unwrap();

	output
		.write_all(&fs::read(bs_bins.join("bootstrapper.bin")).unwrap())
		.unwrap();
	output
		.write_all(&fs::read(bs_bins.join("bootloader.bin")).unwrap())
		.unwrap();
	output
		.write_all(&fs::read(bs_bins.join("elf-loader.bin")).unwrap())
		.unwrap();

	let kernel_path = target
		.join("x86_64-unknown-none")
		.join(profile)
		.join("kernel");
	output.write_all(&fs::read(kernel_path).unwrap()).unwrap();
}
