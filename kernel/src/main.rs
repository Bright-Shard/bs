#![no_std]
#![no_main]

mod vga_text_buffer;

#[no_mangle]
extern "C" fn main() {
    println!("HALLO FROM KERNEL");
}

#[cfg(not(test))]
mod panic {
    use {super::*, core::panic::PanicInfo};

    #[panic_handler]
    fn ohgod(info: &PanicInfo) -> ! {
        println!("\n\n(don't?) PANIC:\n\n{info}");
        loop {}
    }
}
