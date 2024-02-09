#![no_std]
#![no_main]

use core::{arch::global_asm, mem};

/// Where the bootstrapper loads programs into memory.
const LOAD_ADDR: u16 = 0x7E00;

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
    mov $0x7C00, %bp

	/* Jump to Rust */
    call loader
"#,
// We actually need this because you can't do long jumps correctly in the intel
// syntax for some reason
options(att_syntax)
}

#[no_mangle]
extern "C" fn loader() -> ! {
	// Load bootloader into memory
	// It returns the next unread disk sector, which will be the start of
	// the ELF loader
	let elfloader_sector = load_program(1, LOAD_ADDR);

	// Call bootloader
	let main = LOAD_ADDR as *const ();
	let main: extern "C" fn() = unsafe { mem::transmute(main) };
	main();

	// We're now in 64-bit mode and can't use BIOS calls, since they're 16-bit
	// TODO: Write a PCI IDE driver, which can read from disk, and use that to read
	// from disk instead of BIOS. This will give us more control and let us load the
	// ELF loader into memory here.

	loop {}

	panic!()
}

/// Starting at <sector>, reads the disk into memory until it encounters 0xdeadbeef. The data gets
/// loaded to 0x7E00.
fn load_program(mut sector: u64, mut address: u16) -> u16 {
	// The signature found at the end of the current disk sector (used to look for 0xdeadbeef, the end
	// of the bootloader)
	let mut sig = 0u32;

	while sig != 0xdeadbeef {
		common::disks::read_sectors(&mut sector, 1, &mut address, 0, None);

		// Load the sector's signature
		sig = unsafe {
			u32::from_ne_bytes([
				*((address - 4) as *const u8),
				*((address - 3) as *const u8),
				*((address - 2) as *const u8),
				*((address - 1) as *const u8),
			])
		};
	}

	sector as _
}

#[cfg(not(test))]
mod panic {
	use core::{arch::asm, panic::PanicInfo};

	#[panic_handler]
	fn kys(info: &PanicInfo) -> ! {
		unsafe {
			// TODO: Make a cleaner API for this
			let printer = &mut common::printing::GLOBAL_PRINTER;
			printer.clear();

			// QEMU cuts off the top 2 lines of the console on my mac so we
			// add a \n\n to get around it
			printer.write_nofmt("\n\nBOOTSTRAPPER PANIC");
		}

		loop {
			unsafe {
				asm!("hlt");
			}
		}
	}
}
