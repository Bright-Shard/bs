#![no_std]
#![no_main]

use {
	common::printing::Printer,
	core::{
		arch::{asm, global_asm},
		fmt::Write,
		mem,
	},
};

mod disk;

// This is where BS starts. It's written in AT&T syntax because for some reason I
// can't correctly make a long jump in Intel syntax. The rest of the project is in
// the much saner Intel syntax.
global_asm! {
r#"
.section .asm, "awx"
.global asm_main
.code16

asm_main:
    /*
        Clear CS segment
        CS stores the base address of code in memory. We want it to be 0 so it's
        simpler to work with addresses and don't have to worry about a weird offset
        that the BIOS randomly set here. The only way to clear CS is by performing a
        long jump.
    */
    ljmp $0, $call_bootloader

call_bootloader:
    /*
        Clear direction flag
        If the direction flag is set, the CPU reads strings backwards in memory. We
        want it cleared, so the CPU reads strings forwards in memory.
    */
    cld

    /*
        Disable CPU interrupts
        CPU interrupts handle things like keypresses from a keyboard. We don't want to
        worry about these for now.
    */
    cli

    /*
        Zero segment registers
        Segment registers describe the base of some segment of memory - a code segment,
        data segment, etc. These have random values from the BIOS. For simplicity, BS
        sets them all to 0 so everything has the same address it'd have in actual memory.
        The CS (code segment) register was cleared above.
    */
    mov $0, %ax
    mov %ax, %ds
    mov %ax, %es
    mov %ax, %ss
    mov %ax, %fs
    mov %ax, %gs

    /* Set up the stack */
    mov $0x7C00, %sp

	/* Jump to Rust, passing dx as an argument (the `drive` argument in `loader`) */
    push %dx
    // I should only have to push it once, but amazingly, that doesn't work. So we do it twice.
    // Need to look into this more - I'm assuming it's something-something compiler optimisations.
    // rust-osdev's bootloader only has to push it once. They also always compile in release mode.
    push %dx
    call loader
"#,
// We actually need this because you can't do long jumps correctly in the intel
// syntax for some reason
options(att_syntax)
}

#[no_mangle]
extern "C" fn loader(drive: u16) -> ! {
	// Load bootloader into memory
	// It returns the last read sector, aka the end of the bootloader program
	let _end_of_bootloader = disk::load_program(1, drive);

	// Call bootloader
	let main = 0x7E00 as *const ();
	let main: fn() = unsafe { mem::transmute(main) };
	main();

	// We're now in 64-bit mode and can't use BIOS calls, since they're 16-bit
	// TODO: Write a PCI IDE driver, which can read from disk, and use that to read
	// from disk instead of BIOS. This will give us more control and let us load the
	// ELF loader into memory here.

	loop {
		unsafe { asm!("hlt") }
	}
}

#[cfg(not(test))]
mod panic {
	use core::{arch::asm, fmt::Write, panic::PanicInfo};

	#[panic_handler]
	fn kys(_info: &PanicInfo) -> ! {
		// QEMU cuts off the top 2 lines of the console on my mac so we
		common::printing::Printer::get_global()
			.write_str("\n\nBOOTSTRAPPER PANIC")
			.unwrap();

		loop {
			unsafe {
				asm!("hlt");
			}
		}
	}
}
