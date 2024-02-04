#![no_std]
#![no_main]

use {
	acpi::{
		rsdp::{Rsdp, RsdpXsdpError},
		rsdt::Rsdt,
	},
	common::{
		gdt::*,
		interrupts::{Idt, IdtDescriptor, InterruptDescriptor},
		paging::*,
		*,
	},
	core::{arch::asm, mem::ManuallyDrop},
};

#[no_mangle]
#[link_section = ".main"]
extern "C" fn main(sector: u16) -> ! {
	unsafe {
		asm!(
			"mov ah, 0x0e",
			"int 0x10",
			in("al") b'/',
		);
	}
	// TODO: Enable A20 line - https://wiki.osdev.org/A20_Line
	// QEMU has it enabled by default, so we don't need it for now.

	// Enable 64-bit mode
	// https://wiki.osdev.org/Entering_Long_Mode_Directly
	// https://forum.osdev.org/viewtopic.php?f=1&t=11093&sid=e95191d8cf1676df0e60df6853b220d3

	// Structs we need to enter 64-bit mode
	let gdt_descriptor = build_gdt();
	let idt_descriptor = build_idt();
	let page_map_level_4 = build_page_tables();

	// Disable interrupt requests
	unsafe {
		asm!(
			"push ax",
			"mov al, 0xFF",
			"out 0xA1, al",
			"out 0x21, al",
			"nop",
			"nop",
			"pop ax"
		)
	}

	// Load the IDT
	unsafe { asm!("lidt [{}]", in(reg) &idt_descriptor) }

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

	// Load the  page map level 4 (which implicitly loads all page tables, since it points to the other tables)
	unsafe { asm!("mov cr3, eax", in("eax") &page_map_level_4) }

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
	// This, combined with the LME bit above, jumps straight from real/16-bit mode into 64-bit mode
	unsafe {
		asm!(
			"push eax",
			"mov eax, cr0",
			"or eax, 0x01",
			"or eax, 1 << 31",
			"mov cr0, eax",
			"pop eax",
		)
	}

	// Load the GDT
	unsafe { asm!("lgdt [{}]", in(reg) &gdt_descriptor) }

	// Jump back to the bootstrapper at 0x7C00. Now that 64-bit mode is enabled, it'll bootstrap
	// the elf-loader instead of bootloader.
	unsafe { asm!("push ax", "mov ecx, 0x7C00", "jmp ecx", "hlt", in("ax") sector) }

	panic!()
}

/// Builds and sets a GDT with 3 entries: null, all memory read/write, all memory executable.
/// If that sounds unsafe, the real memory permissions will be configured later with paging. x86_64
/// actually doesn't support any other GDT configuration, since it's deprecated and paging is used instead,
/// but we still have to make a GDT to enable it. See the gdt.rs docs for more info.
///
/// This uses `ManuallyDrop` to leak the GDT and prevent it from ever getting destructed.
fn build_gdt() -> ManuallyDrop<GdtDescriptor> {
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

	ManuallyDrop::new(GdtDescriptor {
		size: ((8 * gdt.len()) - 1) as u16,
		offset: &gdt as *const _ as u64,
	})
}

fn build_idt() -> ManuallyDrop<IdtDescriptor> {
	let idt = ManuallyDrop::new(Idt {
		interrupts: [InterruptDescriptor::NULL],
	});
	ManuallyDrop::new(IdtDescriptor {
		offset: &idt as *const ManuallyDrop<Idt<1>> as _,
		size: 0,
	})
}

/// Identity-maps .5tib of memory with all permissions. This amount of memory probably doesn't exist
/// on the actual computer, but that's not important, because that much memory won't be used anyways;
/// this is just an easy way to set RWX permissions for all memory on the machine while the kernel
/// is loaded. The kernel is responsible for detecting the actual amount of memory on the machine and
/// setting actual memory permissions.
///
/// This uses `ManuallyDrop` to leak the pages and prevent them from ever getting destructed.
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
		// Each page maps 1gib of memory
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

// Unused fn. I was adding support for PCIe devices before finding out that the default QEMU
// machine doesn't support PCIe devices. I didn't want to delete the code so it's saved here.
fn _pcie() {
	let mut address = 0;
	let mut maybe_rsdp = None;

	while address < 0xFFFFF {
		let rsdp = unsafe { Rsdp::try_from_raw(address as _) };
		if let Ok(rsdp) = rsdp {
			maybe_rsdp = Some(rsdp);
			break;
		}

		address += 16;
	}
	let Some(rsdp) = maybe_rsdp else {
		panic!("Failed to find RSDP");
	};
	// TODO: Handle XSDP (Extended System Descriptor Pointer)
	// Can use: `if let Ok(xsdp: &Xsdp) = rsdp.try_into() {}`
	// Then need to follow XSDP pointer instead of RSDP pointer

	println!("Found RSDP. {:?}", rsdp.signature);
	let rsdt = unsafe { Rsdt::try_from_raw(rsdp.rsdt_address as _).unwrap() };
	println!("Found RSDT. {:?}", rsdt.descriptor.signature);
	for table in rsdt.tables {
		let rsdt = unsafe { Rsdt::try_from_raw(*table as _).unwrap() };
		println!(
			"    Table in RSDT: {}",
			core::str::from_utf8(&rsdt.descriptor.signature).unwrap()
		);
	}
	let mcfg = rsdt.find_table("MCFG").unwrap();
	println!("OEM ID: {:?}", mcfg.oem_id);
}
