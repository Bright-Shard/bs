#![no_std]
#![no_main]

use core::{
    arch::{asm, global_asm},
    mem::transmute,
    panic::PanicInfo,
};

// Boilerplate ASM that sets up the stack, disables interrupts, zeroes segment registers,
// and calls the main function.
global_asm! {
r#"
.section .asm, "awx"
.global asm_main
.code16

asm_main:
    /* Disable CPU interrupts */
    cli
    
    /* Zero segment registers */
    mov ax, 0
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov fs, ax
    mov gs, ax
    
    /* Set up the stack */
    mov sp, 0x7c00
    
    /* Call main with dx as the argument */
    mov dh, 0
    push dx
    call main
"#
}

#[no_mangle]
extern "C" fn main(disk: u16) -> ! {
    // Where the disk is loaded to in memory
    let mut address: u16 = 0x8000;
    // The sector of the disk to read
    let mut sector: u8 = 2;
    // The signature found at the end of the current disk sector
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
    // from 0x8000 (where we started reading the disk to)
    let main = 0x8000 as *const ();
    let main: fn() -> ! = unsafe { transmute(main) };
    main()
}

#[panic_handler]
fn kys(_: &PanicInfo) -> ! {
    loop {}
}
