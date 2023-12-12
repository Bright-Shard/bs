#![no_std]
#![no_main]

mod elf;
mod gdt;
mod paging;
mod print;

use {
    core::{arch::asm, panic::PanicInfo},
    gdt::*,
    paging::*,
};

#[no_mangle]
#[link_section = ".main"]
fn main() -> ! {
    println!("BS bootloader started.");

    unsafe { asm!("lgdt [{}]", in(reg) &GDT_SIZE) };
    println!("Loaded GDT");

    unreachable!()
}

#[link_section = ".page_table"]
static THING: usize = 0;

/// See the gdt module for an explanation of the GDT. This GDT has 3 entries: The first is null (it's required), the second
/// marks all memory as executable, and the third marks all memory as writable. If this sounds unsafe, it technically is,
/// but x86_64 doesn't actually support memory segmentation anyways - it instead uses memory paging, which is more modern
/// and easier to use - so there's not really another way to set this up.
#[link_section = ".gdt"]
#[allow(dead_code)] // This is actually used, just not directly by Rust
static GDT: [SegmentDescriptor; 3] = [
    [0, 0, 0, 0, 0, 0, 0, 0],
    SegmentDescriptorBuilder {
        base: 0,
        limit: gdt::U20_MAX,
        flags: SegmentFlagsBuilder {
            paged_limit: true,
            protected: false,
            long: true,
        },
        access: SegmentAccessBuilder {
            valid: true,
            privilege: 0,
            non_system: true,
            executable: true,
            direction_conforming: false,
            read_write: true,
            accessed: true,
        },
    }
    .build(),
    SegmentDescriptorBuilder {
        base: 0,
        limit: gdt::U20_MAX,
        flags: SegmentFlagsBuilder {
            paged_limit: true,
            protected: false,
            long: true,
        },
        access: SegmentAccessBuilder {
            valid: true,
            privilege: 0,
            non_system: true,
            executable: false,
            direction_conforming: false,
            read_write: true,
            accessed: true,
        },
    }
    .build(),
];
#[link_section = ".gdt.size"]
static GDT_SIZE: u16 = 8 * 3; // Note: can replace with `addr_of_val` once it's stabilised to avoid hardcoding this

#[panic_handler]
fn kys(_: &PanicInfo) -> ! {
    loop {}
}
