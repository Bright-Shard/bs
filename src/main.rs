#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
fn kys(_: &PanicInfo) -> ! {
    loop {}
}
