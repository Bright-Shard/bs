#![no_std]
#![no_main]

use {
	common::println_nofmt as println,
	core::{
		arch::{asm, global_asm},
		mem::transmute,
	},
};

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
        Check if PAE (Physical Address Extension) is enabled
        PAE should be disabled on boot, so if it's enabled, it means our bootloader
        has run and entered 64-bit mode already. If it's disabled, we load the
        bootloader, which enters 64-bit mode; if it's enabled, we load the elfloader,
        which loads the kernel.
    */
    mov %cr4, %ecx
    and $0b100000, %ecx
    cmp $0b100000, %ecx
    je call_elfloader
    ljmp $0, $call_bootloader

call_elfloader:
    call elfloader

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
        The CS (code segment) register was cleared when jumping to the stage_1_setup
        label; the only way to clear it is to long jump with 0 as the segment.
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

    /*
        Pushing values to the stack lets us pass them as arguments to a fn. DX (in theory)
        holds the disk BS got booted from (some BIOSes may not set this correctly, but
        we'll deal with that later). We hand this to Rust so it can read the rest of
        the bootloader from the same disk.
    */
    mov $0, %dh
    pushw %dx
    call bootloader
"#,
// We actually need this because you can't do long jumps correctly in the intel
// syntax for some reason
options(att_syntax)
}

/// Loads the bootloader into memory and calls it
#[no_mangle]
extern "C" fn bootloader(_disk: u16) -> ! {
	let sector = load_program(1);

	// Because of its link script, the bootloader's main function is at its 0th byte, so we can load it
	// from 0x7E00 (where we started reading the disk to)
	let main = 0x7E00 as *const ();
	let main: extern "C" fn(u16) -> ! = unsafe { transmute(main) };
	main(sector)
}

/// Loads the ELF loader into memory and calls it
#[no_mangle]
extern "C" fn elfloader(sector: u16) -> ! {
	// Note to self:
	// NO PRINTING
	// NO BIOS CALLS
	// NO LONG JUMPS
	unsafe { asm!("hlt") }
	load_program(sector as u64);

	// Because of its link script, the bootloader's main function is at its 0th byte, so we can load it
	// from 0x7E00 (where we started reading the disk to)
	let main = 0x7E00 as *const ();
	let main: extern "C" fn() -> ! = unsafe { transmute(main) };
	main()
}

/// Starting at <sector>, reads the disk into memory until it encounters 0xdeadbeef. The data gets
/// loaded to 0x7E00. This function will then jump to 0x7E00.
fn load_program(mut sector: u64) -> u16 {
	// Where the disk is being loaded to in memory
	let mut address: u16 = 0x7E00;
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
	use {common::println, core::panic::PanicInfo};

	#[panic_handler]
	fn kys(info: &PanicInfo) -> ! {
		println!("Bootstrapper panic");
		loop {}
	}
}
