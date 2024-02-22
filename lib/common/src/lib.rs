#![no_std]

pub mod gdt;
pub mod interrupts;
pub mod paging;
pub mod printing;

#[cfg(all(not(test), feature = "panic"))]
mod panic {
	use {super::*, core::panic::PanicInfo};

	#[panic_handler]
	fn ohgod(info: &PanicInfo) -> ! {
		println!("\n\n(don't?) PANIC:\n\n{info}");
		loop {}
	}
}
