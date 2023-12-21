#![no_std]
#![no_main]

mod disks;
#[allow(dead_code, clippy::upper_case_acronyms)]
mod elf;
mod gdt;
mod paging;
mod print;

use {
    core::{arch::asm, mem::ManuallyDrop},
    gdt::*,
    paging::*,
};

#[no_mangle]
#[link_section = ".main"]
extern "C" fn main(sector: u8) -> ! {
    elf::parse_from_sector(sector);
    let gdt_descriptor = build_gdt();
    let page_map_level_4 = build_page_tables();
    enable_64_bit_mode(&gdt_descriptor, &page_map_level_4);
    // We can't use BIOS calls now that we're in 64-bit mode. This writes to the VGA screen buffer instead.
    // Unfortunately, this does mean we lose our place and start printing from the top of the screen...
    "64-bit mode enabled. Booting kernel..."
        .bytes()
        .enumerate()
        .for_each(|(idx, byte)| {
            let byte = u16::from_be_bytes([0b0000_0111, byte]);
            unsafe {
                asm!(
                    "mov [eax], ecx",
                    in("eax") 0xb8000 + (idx * 2),
                    in("ecx") byte
                )
            }
        });
    unsafe {
        asm!("hlt");
    }

    panic!()
}

/// Builds and sets a GDT with 3 entries: null, all memory read/write, all memory executable.
/// If that sounds unsafe, the real memory permissions will be configured later with paging. x86_64
/// actually doesn't support any other GDT configuration, since it's deprecated and paging is used instead,
/// but we still have to make a GDT to enable it. See the gdt.rs docs for more info.
///
/// This uses `ManuallyDrop` to prevent the GDT from ever getting destructed.
fn build_gdt() -> ManuallyDrop<GDTDescriptor> {
    let gdt = ManuallyDrop::new([
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
    ]);

    ManuallyDrop::new(GDTDescriptor {
        offset: &gdt as *const _ as u64,
        size: ((8 * gdt.len()) - 1) as u16,
    })
}

/// Identity-maps .5tib of memory with all permissions. This amount of memory probably doesn't exist
/// on the actual computer, but that's not important, because that much memory won't be used anyways;
/// this is just an easy way to set RWX permissions for all memory on the machine while the kernel
/// is loaded. The kernel is responsible for detecting the actual amount of memory on the machine and
/// setting actual memory permissions. See the paging.rs docs for more info.
///
/// This uses `ManuallyDrop` to prevent the pages from ever getting destructed.
fn build_page_tables() -> PageMap {
    // Build the page directory pointer table
    let mut page_directory_pointer_table = PageMap::new();
    let mut address = 0;
    for entry in page_directory_pointer_table.0.iter_mut() {
        *entry = PageDirectoryPointerTableEntryBuilder {
            present: true,
            writable: true,
            user_mode: false,
            write_through: false,
            cache_disabled: false,
            accessed: false,
            dirty: false,
            direct_map: true,
            global: false,
            pat: false,
            address,
            protection_key: None,
            execute_disable: false,
        }
        .build();
        // Each one pages 1gib
        address += 0x40000000;
    }

    // Build the page map level 4 and point it to the page directory pointer table
    let mut page_map_level_4 = PageMap::new();
    page_map_level_4.0[0] = PageMapLevel4EntryBuilder {
        present: true,
        writable: true,
        user_mode: false,
        write_through: false,
        cache_disabled: false,
        accessed: false,
        address: &page_directory_pointer_table as *const _ as u64,
        execute_disable: false,
    }
    .build();

    page_map_level_4
}

/// Enables 64-bit mode and loads the GDT/page tables. Normally, 32-bit mode must be enabled first,
/// but the OSDev wiki documents a method to jump straight from 16-bit mode to 64-bit mode, which this uses:
/// https://wiki.osdev.org/Entering_Long_Mode_Directly
fn enable_64_bit_mode(gdt_descriptor: &ManuallyDrop<GDTDescriptor>, page_map_level_4: &PageMap) {
    // Load the GDT
    unsafe { asm!("lgdt [{}]", in(reg) &gdt_descriptor) }

    // Load the  page map level 4 (which implicitly loads all page tables, since it points to the other tables)
    unsafe { asm!("mov cr3, eax", in("eax") &page_map_level_4.0) }

    // Sets the PAE bit/enables PAE. PAE: Physical Address Extension, allowing access to >4gb of memory.
    // This is required to enter 64-bit mode.
    unsafe {
        asm!(
            "push eax",
            "mov eax, cr4",
            "or eax, (1 << 5)",
            "mov cr4, eax",
            "pop eax"
        )
    }

    // Set the EFER MSR's LME bit.
    // MSR: Model-specific registers - registers that can change between CPU models. Technically you should
    //      check if an MSR is available with CPUID before using them, but BS only supports x86_64 processors,
    //      and this MSR in particular is always present for those.
    // EFER: An MSR with lots of settings related to 64-bit mode, syscalls, and more.
    // LME: Long Mode Enable. The bit in the EFER register that enables long mode (aka 64-bit mode).
    //
    // MSRs are all identified by specific numbers. To read an MSR, call `rdmsr` and provide the MSR's number
    // in ECX. The value will be read into EAX. To write an MSR, call `wrmsr` with the MSR's number in ECX and
    // the value to write in EAX.
    unsafe {
        asm!(
            "push eax",
            "push ecx",
            "mov ecx, 0xC0000080", // The EFER MSR's number
            "rdmsr",
            "or eax, 0x00000100", // The LME bit
            "wrmsr",
            "pop ecx",
            "pop eax",
        )
    }

    // Enable paging and protected mode simultaneously
    unsafe {
        asm!(
            "push eax",
            "mov eax, cr0",
            "or eax, 0x80000001",
            "mov cr0, eax",
            "pop eax"
        )
    }
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
