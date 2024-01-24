#![no_std]
#![no_main]

use common::*;

#[no_mangle]
extern "C" fn main() {
	// Kernel just has a hello world for now; when I see this message I know
	// Frieren is working her magic.
	println!("HALLO FROM KERNEL");
}
