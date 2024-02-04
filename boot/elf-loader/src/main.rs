#![no_std]
#![no_main]

use common::*;
use core::arch::asm;

#[no_mangle]
#[link_section = ".main"]
extern "C" fn main() -> ! {
	println!("Inside 64-bit ELF loader :3");
	unsafe { asm!("hlt") }
	unreachable!()
}
