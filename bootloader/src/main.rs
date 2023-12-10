#![no_std]
#![no_main]

use core::{
    arch::{asm, global_asm},
    panic::PanicInfo,
};

static MESSAGE: &str = "Hello, world!";

global_asm! {
r#"
.section .asm, "awx"
.global asm_main
.code16

asm_main:
    cld
    call main
"#
}

#[no_mangle]
extern "C" fn main() {
    print(&"Hello from rust!");
    print(MESSAGE);
}

fn print(text: &str) {
    text.bytes().for_each(|letter| unsafe {
        asm!(
            "mov ah, 0x0e",
            "int 0x10",
            in("al") letter
        );
    });
}

#[panic_handler]
fn kys(_: &PanicInfo) -> ! {
    loop {}
}
