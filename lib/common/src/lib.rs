#![no_std]

#[cfg(feature = "bios")]
pub mod disks;
pub mod gdt;
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
