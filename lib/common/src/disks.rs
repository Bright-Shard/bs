//! Works with BIOS disk services to read data from the disk. It uses LBA (Logical Block Addressing).
//!
//! Resources:
//! - https://wiki.osdev.org/Disk_access_using_the_BIOS_(INT_13h)#LBA_in_Extended_Mode
//! - https://en.wikipedia.org/wiki/Logical_block_addressing#CHS_conversion

use core::arch::asm;

/// Used in LBA addressing to determine what parts of a disk to
/// load and where to load them to.
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
	/// The LBA to read - AKA, the sector to read. This is a 48-bit value, but
	/// takes up 8 bytes of space for whatever reason.
	pub lba: u64,
}

/// Uses LBA to read disk sectors into memory. Reads <sectors> sectors, starting
/// at <lba>, to <segment>:<address> in memory. Will increment <lba> and <address> automatically.
pub fn read_sectors(
	lba: &mut u64,
	sectors: u16,
	address: &mut u16,
	segment: u16,
	disk: Option<u8>,
) {
	let dap = DiskAddressPacket {
		size: 16,
		reserved: 0,
		sectors,
		segment,
		offset: *address,
		lba: *lba,
	};
	unsafe {
		asm!("pusha", "mov si, ax", "mov ah, 0x42", "int 0x13", "popa", in("ax") &dap, in("dl") disk.unwrap_or(0x80_u8));
	}

	*lba += sectors as u64;
	*address += sectors * 512;
}
