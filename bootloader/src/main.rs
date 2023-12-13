#![no_std]
#![no_main]

mod elf;
mod gdt;
mod paging;
mod print;

use {
    core::{arch::asm, mem::ManuallyDrop, panic::PanicInfo},
    gdt::*,
    paging::*,
};

#[no_mangle]
#[link_section = ".main"]
fn main() -> ! {
    println!("BS bootloader started.");

    build_gdt();
    println!("Loaded GDT");

    unreachable!()
}

/// Builds and sets a GDT with 3 entries: null, all memory read/write, all memory executable.
/// If that sounds unsafe, the real memory permissions will be configured later with paging. x86_64
/// actually doesn't support any other GDT configuration, since it's deprecated and paging is used instead,
/// but we still have to make a GDT to enable it.
///
/// See the gdt module docs for more info.
fn build_gdt() {
    let gdt = [
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
                present: true,
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
                present: true,
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

    // ManuallyDrop prevents the descriptor from ever being deallocated. This is what we want,
    // because the descriptor is essentially a static that's been defined a runtime - the GDT
    // should *always* exist. It'd technically be possible to get the GDT's address at compile-time
    // with linker scripts; however, this takes some of the code out of rust, which makes it hard to read.
    let gdt_descriptor = ManuallyDrop::new(GDTDescriptor {
        offset: (&gdt as *const _ as usize) as u64,
        size: ((8 * gdt.len()) - 1) as u16,
    });

    unsafe { asm!("lgdt [{}]", in(reg) &gdt_descriptor) };
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
