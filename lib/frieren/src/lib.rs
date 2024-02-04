#![no_std]

pub mod structs;
pub use structs::*;

use core::mem;

/// Frieren failed to cast a spell
pub enum ElfError {
	/// Couldn't find the magic bytes in the ELF file
	NoMagicBytes,
	/// ELF is 32-bit, not 64-bit
	/// TODO: There's probably a way to use 32-bit compat...
	Bitness32,
	/// ELF's endianness didn't match the native endianness
	/// TODO: Is this actually an error? Maybe there's compat
	BadEndianness,
	/// The ABI wasn't SystemV
	BadABI,
	/// The ELF wasn't v1
	BadVersion,
	/// The reported size of a header in the file header didn't match the size of our structs
	/// (ie `FileHeader.size` != `mem::size_of::<FileHeader>()`).
	BadHeaderSize(Header),
}

pub enum Header {
	Section,
	Program,
	File,
}

impl FileHeader {
	/// Takes a raw pointer to a file header, verifies its contents, and errors if anything is wrong.
	///
	/// # Safety
	/// - `ptr` must be a non-null, aligned pointer
	/// - `ptr` must live for at least `'a`
	pub unsafe fn try_from_raw<'a>(ptr: *const FileHeader) -> Result<&'a Self, ElfError> {
		let header = unsafe { &*ptr };

		Err(if header.magic_bytes != [0x7F, 0x45, 0x4C, 0x46] {
			ElfError::NoMagicBytes
		} else if header.bitness != Bitness::X64 {
			ElfError::Bitness32
		} else if header.endianess != Endianess::NATIVE {
			ElfError::BadEndianness
		} else if header.elf_version != 1 || header.header_version != 1 {
			ElfError::BadVersion
		} else if header.abi != ABI::SystemV {
			ElfError::BadABI
		} else if header.section_header_size != mem::size_of::<SectionHeader>() as u16 {
			ElfError::BadHeaderSize(Header::Section)
		} else if header.program_header_size != mem::size_of::<ProgramHeader>() as u16 {
			ElfError::BadHeaderSize(Header::Program)
		} else if header.size != mem::size_of::<FileHeader>() as u16 {
			ElfError::BadHeaderSize(Header::File)
		} else {
			return Ok(header);
		})
	}

	/// Returns the (inclusive start, exclusive end) range that holds the section table.
	pub fn section_table_range(&self) -> (usize, usize) {
		let start = self.section_table_offset as usize;
		let len = mem::size_of::<SectionHeader>() * self.section_table_entries as usize;

		(start, start + len)
	}
}

// Old code, just here for when I implement ELF loading

pub fn parse_from_sector(_sector: u8) {
	// let mut sector = (sector - 1) as u64;
	// let base_address = 0xFFFF;
	// let mut address = base_address;
	// disks::read_sectors(&mut sector, 1, &mut address);
	// let header = unsafe { &(address as *const FileHeader).read() };
	// // Check that the file is a supported ELF file
	// if header.magic_bytes != [0x7F, 0x45, 0x4C, 0x46] {
	// 	panic!("Couldn't find the ELF magic bytes.",)
	// }
	// if header.bitness != Bitness::X64 {
	// 	panic!("BS only supports 64-bit ELFs.")
	// }
	// if header.endianess != Endianess::NATIVE {
	// 	todo!("Figure out non-native endianness")
	// }
	// if header.elf_version != 1 || header.header_version != 1 {
	// 	panic!("BS only supports v1 ELFs.",)
	// }
	// if header.abi != ABI::SystemV {
	// 	panic!("BS only supports the SystemV ABI.",)
	// }
	// let object_type = header.object_type;
	// if object_type != ObjectType::Dyn {
	// 	panic!("BS only supports position-independent ELF objects.",)
	// }
	// // Load the section header table
	// let section_table_base = header.section_table_offset as u16;
	// let section_table_len = header.section_header_size * header.section_table_entries;
	// let section_table_end = section_table_base + section_table_len;
	// if section_table_end > 512 {
	// 	let sectors_to_read = section_table_end.div_ceil(512);
	// 	disks::read_sectors(&mut sector, sectors_to_read as _, &mut address);
	// }
	// let section_header_table = unsafe {
	// 	slice::from_raw_parts(
	// 		(base_address + section_table_base) as *const SectionHeader,
	// 		section_table_len as usize,
	// 	)
	// };
	// // Find the file size from the ELF sections
	// let mut total_size = 0;
	// for section_header in section_header_table {
	// 	total_size += section_header.size;
	// }
	// println!("ELF file size: {total_size}");
	// let sectors_to_read = total_size.div_ceil(512);
	// disks::read_sectors(&mut sector, sectors_to_read as _, &mut address);
}
