#![no_std]
#![feature(abi_x86_interrupt)]

pub mod interrupts;
pub mod kbhandler;
pub mod vga;

pub mod prelude {
    pub use super::vga::{print, println};
}

pub fn init() {
    interrupts::IDT.call_once(interrupts::init_idt);
    interrupts::IDT.get().unwrap().load();
    unsafe { interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}
