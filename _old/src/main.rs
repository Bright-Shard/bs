#![no_std]
#![no_main]
use {
    bootloader::{entry_point, BootInfo},
    core::ffi::c_void,
    core::panic::PanicInfo,
    os::prelude::*,
};
entry_point!(main);

fn main(boot_info: &'static BootInfo) -> ! {
    os::init();
    println!("Welcome to the very fancy and modern computer system! Type help for help.");

    os::hlt_loop();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    os::hlt_loop();
}
