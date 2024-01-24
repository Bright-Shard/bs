#![no_std]
#![no_main]

use core::{
	arch::{asm, global_asm},
	mem::transmute,
};

// Performs a long jump to the actual ASM start. As far as I know, this is the only way
// to zero the code segment register.
global_asm! {
r#"
.section .asm, "awx"
.global asm_main
.code16

asm_main:
    ljmp $0, $asm_code
"#,
options(att_syntax)
}

// Boilerplate ASM that sets up the stack, disables interrupts, zeroes segment registers,
// and calls the main function.
global_asm! {
r#"
.section .asm, "awx"
.global asm_code
.code16

asm_code:
    /*
        Fix direction flag (for working with strings)
        From my understanding, if the direction bit is set, instructions that work with
        strings will go backwards in memory instead of forwards. cld clears the direction
        bit so instructions that work with strings go forwards instead.
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
        Segment registers can be used as offsets into memory. It's easier to work with
        memory if these are all 0 instead of whatever random value the BIOS left.
    */
    mov ax, 0
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov fs, ax
    mov gs, ax
    
    /* Set up the stack (see the README for the memory layout of BS' bootsector) */
    mov sp, 0x7C00
    
    /*
        Pushing values to the stack apparently passes them as arguments. DX (in theory)
        holds the disk BS got booted from (some BIOSes may not set this correctly, but
        we'll deal with that later). We hand this to Rust so it can read the rest of
        the bootloader from the same disk.
    */
    mov dh, 0
    push dx
    call main
"#
}

#[no_mangle]
extern "C" fn main(disk: u16) -> ! {
	// Where the disk is being loaded to in memory
	let mut address: u16 = 0x7E00;
	// The next sector of the disk to read
	let mut sector: u8 = 2;
	// The signature found at the end of the current disk sector (used to look for 0xdeadbeef, the end
	// of the bootloader)
	let mut sig = 0u32;

	while sig != 0xdeadbeef {
		unsafe {
			asm!(
				"pusha", // We have to backup/restore CPU registers or Rust will break
				"mov bx, ax", // Address to read disk to (in this case, Rust can't write directly to bx so we write indrectly)
				"mov al, 1", // Number of sectors to read
				"mov ch, 0", // Cylinder to read
				"mov dh, 0", // Head to read
				"mov ah, 2", // Use the read disk function in the disk syscall
				"int 0x13", // Disk syscall
				"popa",
				in("ax") address, // Indirectly write address to bx
				in("cl") sector, // Disk sector to read
				in("dx") disk // Disk to read (should technically be dl but we set dh anyways)
			)
		}

		// Load the sector's signature
		sig = unsafe {
			u32::from_ne_bytes([
				*((address + 508) as *const u8),
				*((address + 509) as *const u8),
				*((address + 510) as *const u8),
				*((address + 511) as *const u8),
			])
		};

		address += 512;
		sector += 1;
	}

	// Because of its link script, the bootloader's main function is at its 0th byte, so we can load it
	// from 0x7E00 (where we started reading the disk to)
	let main = 0x7E00 as *const ();
	let main: extern "C" fn(u8) -> ! = unsafe { transmute(main) };
	main(sector)
}

#[cfg(not(test))]
mod panic {
	use core::panic::PanicInfo;

	#[panic_handler]
	fn ohgod(_info: &PanicInfo) -> ! {
		loop {}
	}
}
