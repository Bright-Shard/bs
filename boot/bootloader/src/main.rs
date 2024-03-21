#![no_std]
#![no_main]

use {
	acpi::{rsdp::Rsdp, rsdt::Rsdt},
	ata::IdeController,
	common::{gdt::*, paging::*, printing::Printer, *},
	core::{
		arch::asm,
		mem::{ManuallyDrop, MaybeUninit},
	},
	pci::{
		classification::{Class, HeaderType, MassStorageControllerSubclass},
		PciDevice,
	},
};

#[no_mangle]
#[link_section = ".boot-program-main"]
fn main() {
	Printer::get_global().clear();
	// For some reason QEMU cuts off the first 2 lines of the console on my mac; seeing this
	// message just confirms prints aren't getting cut off.
	println!("\n\nhewwo");

	// Eventually this PCI code is going to get put in its own crate/boot program.
	// Right now it's here as a POC.
	println!("PCI");
	pci();
	println!("ICP");

	// TODO: Enable A20 line - https://wiki.osdev.org/A20_Line
	// QEMU has it enabled by default, so we don't need it for now.

	// Enable 64-bit mode
	// https://wiki.osdev.org/Entering_Long_Mode_Directly
	// https://forum.osdev.org/viewtopic.php?f=1&t=11093&sid=e95191d8cf1676df0e60df6853b220d3

	// Structs we need to enter 64-bit mode
	let gdt_descriptor = build_gdt();
	let page_map_level_4 = build_page_tables();

	// Sets the PAE bit/enables PAE. PAE: Physical Address Extension, allowing access to >4gb of memory.
	// This is required to enter 64-bit mode.
	// TODO: Investigate: PAE seems to break under QEMU, but OSDev Wiki claims it's needed for 64-bit mode.
	// println!("Enabling PAE & PGE");
	// unsafe {
	// 	asm!(
	// 		"mov eax, cr4",
	// 		"or eax, (1 << 5)",
	// 		"mov cr4, eax",
	// 		out("eax") _
	// 	)
	// }

	// Load the page map level 4 (PML4)
	// The PML4 is the top-level page table, and its entries point to lower level page tables
	// Thus this implicitly loads all our page tables
	println!("Loading PML4");
	unsafe { asm!("mov cr3, eax", in("eax") (page_map_level_4.ptr() as u32)) }

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
	println!("Setting LME");
	unsafe {
		asm!(
			"mov ecx, 0xC0000080", // The EFER MSR's number
			"rdmsr",
			"or eax, 1 << 8", // The LME bit
			"wrmsr",
			// Tell rust we use these registers
			out("eax") _,
			out("ecx") _
		)
	}

	// Enable paging and protected mode simultaneously
	// This, combined with what we did above, jumps straight from real/16-bit mode into 64-bit mode
	println!("Enabling paging & protected mode");
	unsafe {
		asm!(
			"mov eax, cr0",
			"or eax, 1 << 0",
			"or eax, 1 << 16",
			"or eax, 1 << 31",
			"mov cr0, eax",
			// Tell rust we use this register
			out("eax") _
		)
	}

	// Load the GDT
	// The GDT is the legacy way for defining memory permissions, from before paging was invented
	// The CPU will actually ignore this in 64-bit mode and use pages instead
	// However, it's still required to set up a GDT to leave 16-bit mode
	println!("Loading GDT");
	unsafe { asm!("lgdt [{}]", in(reg) &gdt_descriptor) }
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

/// Identity-maps 2mib of memory with RWX permissions. This is temporary, just enough to get our kernel booted.
///
/// This uses `ManuallyDrop` to leak the pages and prevent them from ever getting destructed.
fn build_page_tables() -> ManuallyDrop<PageMap<PageMapLevel4Entry>> {
	let mut page_table = ManuallyDrop::new(PageMap::<PageTableEntry>::new());
	let mut address = 0;
	for entry in page_table.iter_mut() {
		entry
			.set_present(true)
			.set_writable(true)
			.set_address(address);
		address += 0x1000;
	}

	let mut page_directory = ManuallyDrop::new(PageMap::<PageDirectoryEntry>::new());
	page_directory[0]
		.set_present(true)
		.set_writable(true)
		.set_address(page_table.ptr() as _);

	let mut page_directory_pointer_table =
		ManuallyDrop::new(PageMap::<PageDirectoryPointerTableEntry>::new());
	page_directory_pointer_table[0]
		.set_present(true)
		.set_writable(true)
		.set_address(page_directory.ptr() as _);

	let mut page_map_level_4 = ManuallyDrop::new(PageMap::<PageMapLevel4Entry>::new());
	page_map_level_4[0]
		.set_present(true)
		.set_writable(true)
		.set_address(page_directory_pointer_table.ptr() as _);

	page_map_level_4
}

// PCI will eventually be put in its own boot program so the bootstrapper can use it to read from
// disk. Right now it's here as a POC.
fn pci() {
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

	println!("Found RSDP at {address:#x}",);
	let rsdt = unsafe { Rsdt::try_from_raw(rsdp.rsdt_address as _).unwrap() };
	let address = rsdp.rsdt_address;
	println!("Found RSDT at {address:#x}");
	for table in rsdt.tables {
		let rsdt = unsafe { Rsdt::try_from_raw(*table as _).unwrap() };
		println!(
			"    Table in RSDT: {}",
			core::str::from_utf8(&rsdt.descriptor.signature).unwrap()
		);
	}

	// If the system supports PCIe, there will be an MCFG table. Otherwise, we fall back to using regular PCI.
	if let Some(_mcfg) = rsdt.find_table("MCFG") {
		todo!("PCIe")
	} else {
		println!("No PCIe detected, falling back on PCI...");

		// PCI bus 0, device 0, fn 0 is the root PCI bridge
		let Some(root) = PciDevice::new(0, 0, 0) else {
			panic!("Failed to initialise PCI :c")
		};

		handle_pci_bridge(root);
	}
}

fn handle_pci_bridge(mut bridge: PciDevice) {
	let header = bridge.header().unwrap();

	if header.multi_function {
		let bus = bridge.bus();
		let device = bridge.device();
		let mut function = 0;
		while let Some(mut bridge) = PciDevice::new(bus, device, function) {
			let register = bridge.read_register(6).unwrap();
			let bus = register[1];
			handle_pci_bus(bus);

			function += 1;
		}
	} else {
		let register = bridge.read_register(6).unwrap();
		let bus = register[2];
		handle_pci_bus(bus);
	}
}

fn handle_pci_bus(bus: u8) {
	for device_id in 0..32 {
		if let Some(mut device) = PciDevice::new(bus, device_id, 0) {
			let header = device.header().unwrap();

			if header.kind == HeaderType::PciToPci {
				println!("PCI bridge at {bus}.{device_id}");
				handle_pci_bridge(device);
			} else if header.multi_function {
				let bus = device.bus();
				let device = device.device();
				let mut function = 0;
				while let Some(mut device) = PciDevice::new(bus, device, function) {
					handle_pci_device(&mut device);
					function += 1;
				}
			} else {
				handle_pci_device(&mut device);
			}
		}
	}
}

fn handle_pci_device(device: &mut PciDevice) {
	println!("Found PCI device with class: {:?}", device.class());
	if device.class()
		== Some(Class::MassStorageController(
			MassStorageControllerSubclass::Ide,
		)) {
		let mut controller = IdeController::from_pci(device).unwrap();
		controller.primary_channel.set_interrupts(false);
		controller.secondary_channel.set_interrupts(false);
		println!(
			"Found IDE controller. prog_if: {:#b}",
			device.programming_interface().unwrap()
		);

		controller.primary_channel.set_disk(ata::IdeDisk::Primary);
		controller
			.primary_channel
			.send_command(ata::AtaCommand::ReadPio, 0, 0)
			.unwrap();
		let mut output: [u16; 256] = [0; 256];
		for part in output.iter_mut() {
			*part = controller
				.primary_channel
				.read_register(ata::AtaRegister::Data);
		}
		print!("First sector on drive: [");
		for word in output {
			for byte in word.to_ne_bytes() {
				print!("{byte:02x}, ")
			}
		}
		println!("]")
	}
}
