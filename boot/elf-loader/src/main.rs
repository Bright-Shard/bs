#![no_std]
#![no_main]

use common::*;
use core::arch::{asm, global_asm};

global_asm! {
r#"
.section .boot-program-main, "awx"
.global asm_main

asm_main:
    call main
"#
}

#[no_mangle]
extern "C" fn main() -> ! {
	println!("\n\nInside 64-bit ELF loader :3");
	unsafe { asm!("hlt") }
	unreachable!()
}
