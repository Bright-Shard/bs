#![no_std]
#![no_main]

use core::{arch::global_asm, panic::PanicInfo};

global_asm! {
r#"
.section .boot, "awx"
.global asm_boot
.code16

asm_boot:
    cld
    mov al, 'a'
    mov ah, 0x0e
    int 0x10
"#
}

#[panic_handler]
fn kys(_: &PanicInfo) -> ! {
    loop {}
}
