//! Works with BIOS disk services to read data from the disk. It uses LBA (Logical Block Addressing).
//!
//! Resources:
//! - https://wiki.osdev.org/Disk_access_using_the_BIOS_(INT_13h)#LBA_in_Extended_Mode
//! - https://en.wikipedia.org/wiki/Logical_block_addressing#CHS_conversion

use core::arch::asm;

/// Reads from `disk`, starting at `start_sector`, until it finds the bytes `0xDEADBEEF`, which
/// mark the end of a BS boot program.
///
/// This uses BIOS' int 13h command to read from disk; see the resources in the module-level docs.
pub fn load_program(start_sector: u64, disk: u16) -> u64 {
	let mut dap = DiskAddressPacket {
		size: 16,
		reserved: 0,
		sectors: 1,
		segment: 0,
		offset: 0x7E00,
		lba: start_sector,
	};

	loop {
		unsafe {
			asm!("pusha", "mov si, ax", "mov ah, 0x42", "int 0x13", "popa", in("ax") &dap, in("dx") disk);
		}

		let signature_bytes = unsafe { *((dap.offset + 508) as *const [u8; 4]) };
		let signature = u32::from_ne_bytes(signature_bytes);
		if signature == 0xDEADBEEF {
			break;
		}

		dap.offset += 512;
		dap.lba += 1;
	}

	dap.lba
}

/// Used in LBA addressing to specify a part of a disk to read and where to read it to in memory.
#[repr(packed)]
pub struct DiskAddressPacket {
	/// The size of this packet. Should be 16, for 16 bytes.
	pub size: u8,
	/// A reserved byte - always 0
	pub reserved: u8,
	/// How many sectors to read from the disk - some BIOSes cap this to 127
	pub sectors: u16,
	/// An offset, starting at <segment>, to the memory address the disk data should be loaded to.
	pub offset: u16,
	/// A memory segment where the disk data will be loaded to. It'll specifically be loaded to <segment> + <offset>.
	pub segment: u16,
	/// The LBA to read - AKA, the sector to read. This is a 48-bit value, but has padding after it so it
	/// ends up being 8 bytes.
	pub lba: u64,
}
