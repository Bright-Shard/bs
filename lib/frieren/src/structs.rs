//! This defines tructures in ELF files. This includes the file header, program header, and section header.
//! This also defines several enums present in those headers.

/// The first few bytes of an ELF file. Contains general file information. Note that this structure
/// looks somewhat different for 32-bit ELFs.
#[repr(packed)]
pub struct FileHeader {
	// This is technically in the identifier, a substructure in the header,
	// but having all of these inside another field is annoying to work with.
	/// Should be 0x7F, then `ELF` in ASCII (7F 45 4C 46).
	pub magic_bytes: [u8; 4],
	/// If this file is 32-bit or 64-bit.
	pub bitness: Bitness,
	/// If this file is little endian or big endian.
	pub endianess: Endianess,
	/// The version of the ELF header - should be 1 for the current version.
	pub header_version: u8,
	/// The ABI this file targets.
	pub abi: ABI,
	/// The version of the ABI this file targets.
	pub abi_version: u8,
	/// Unused bytes. I think this is here for alignment.
	pub padding: [u8; 7],

	// Back to header fields
	/// The type of this ELF file - library, executable, etc.
	pub object_type: ObjectType,
	/// The targeted instruction set. There's so many values here, I didn't bother
	/// making an enum for it.
	pub instruction_set: u16,
	/// The version of this ELF file - should be 1 for the current version.
	pub elf_version: u8,
	/// An offset to the entry point of this ELF file.
	pub entry_point: u64,
	/// An offset to the program header table of this ELF file.
	pub program_table_offset: u64,
	/// An offset to the section header table of this ELF file.
	pub section_table_offset: u64,
	/// Architecture-specific flags.
	pub flags: u32,
	/// The size of this ELF header. It ironically comes after all the fields that change size,
	/// meaning its location in the structure changes... based on the structure's size.
	pub size: u16,
	/// The size of a program header.
	pub program_header_size: u16,
	/// The number of entries in the program header table.
	pub program_table_entries: u16,
	/// The size of a section header.
	pub section_header_size: u16,
	/// The number of entries in the section header table.
	pub section_table_entries: u16,
	/// The index into the section header table that has section names.
	pub section_names_index: u16,
}

/// Each program header describes a segment of an ELF file. These are only needed for executables
/// and shared objects. A segment contains one or more sections.
#[repr(packed)]
pub struct ProgramHeader {
	/// Defines the type for this segment.
	pub program_type: ProgramType,
	/// Permissions for this segment. 1 = execute, 2 = write, 4 = read.
	pub flags: u32,
	/// Where the actual segment is in the file.
	pub offset: u64,
	/// Where the segment should be loaded in memory.
	pub address: u64,
	/// The segment's physical address, if that's relevant.
	pub physical_address: u64,
	/// The size of the segment in the file.
	pub file_size: u64,
	/// The size of the segment in memory. If this is greater than the size of the segment in the
	/// file, the rest of the space should be allocated for this segment anyways and filled with 0s.
	pub memory_size: u64,
	/// The alignment of this segment in memory. 0 or 1 equals no alignment. Otherwise the value
	/// should be positive and a power of 2, and then `address` should equal `offset % alignment`.
	pub alignment: u64,
}

/// Each section header describes a section of the ELF file.
#[repr(packed)]
pub struct SectionHeader {
	/// An offset into the string table, representing this section's name.
	pub name_offset: u32,
	/// The type of this section.
	pub section_type: SectionType,
	/// Flags for this section.
	pub flags: u64,
	/// If this section should be loaded in memory, the address it should be loaded to.
	pub address: u64,
	/// An offset to the section in the ELF file.
	pub offset: u64,
	/// The size of the section in the ELF file (can be 0).
	pub size: u64,
	/// What this is depends on the section type.
	pub link: u32,
	/// What this is depends on the section type.
	pub info: u32,
	/// The alignment of this section in memory. 0 or 1 equals no alignment. Otherwise the value
	/// should be positive and a power of 2, and `address` should be aligned to this value.
	pub alignment: u64,
	/// If this section has fixed-size entries, this contains the size of each entry. Otherwise,
	/// this is 0.
	pub entry_size: u64,
}

/// The type of a program header in the ELF file.
#[repr(u32)]
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum ProgramType {
	/// An unused segment.
	Null = 0,
	/// A loadable segment. These must be loaded into memory.
	Load = 1,
	/// Info for dynamic linking.
	Dynamic = 2,
	/// Contains the path to an interpreter for the program.
	Interpreter = 3,
	/// Generic information.
	Note = 4,
	/// Reserved. Sections with this type don't conform to the ABI.
	Lib = 5,
	/// A segment with the program header table.
	ProgramHeader = 6,
	/// For thread-local storage.
	ThreadLocal = 7,
	// Others are OS/processor specific
}

/// The type of a section header in the ELF file.
#[repr(u32)]
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum SectionType {
	/// Unused.
	Null = 0,
	/// Information defined by and for the program.
	ProgramData = 1,
	/// The symbol table.
	SymbolTable = 2,
	/// The string table, which holds all of the text in the ELF.
	StringTable = 3,
	/// Holds relocation entries with explicit addends.
	RelocationsAddend = 4,
	/// A symbol hash table.
	HashTable = 5,
	/// Information for dynamic linking.
	Dynamic = 6,
	/// Information that marks the file in some way.
	Note = 7,
	/// Just like `ProgramData`, except it holds no data in the actual file.
	NoBits = 8,
	/// Holds relocation entries without explicit addends.
	Relocations = 9,
	/// Reserved. Sections with this type don't conform to the ABI.
	Lib = 10,
	/// Similar to `SymbolTable`, but with less symbols - just the ones needed
	/// for dynamic linking.
	DynamicSymbols = 11,
	// Others are program/processor specific
}

/// If an ELF file is 32-bit or 64-bit.
#[repr(u8)]
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Bitness {
	X32 = 1,
	X64 = 2,
}

/// If an ELF file is little endian or big endian.
#[repr(u8)]
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Endianess {
	Little = 1,
	Big = 2,
}
impl Endianess {
	#[cfg(target_endian = "little")]
	pub const NATIVE: Self = Self::Little;

	#[cfg(target_endian = "big")]
	pub const NATIVE: Self = Self::Big;
}

/// The ELF file's type.
#[repr(u16)]
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum ObjectType {
	None = 0,
	/// I'm not sure, but think this is for compiler intermediaries.
	Relocatable = 1,
	/// A normal, executable program.
	Exectuable = 2,
	/// Wikipedia describes this as a shared object. `readelf` uses this for position-independent code.
	/// Maybe it represents both. The spec is unfortunately quite vague.
	Dyn = 3,
	/// Unsure what core means.
	Core = 4,
	// Other values are OS-specific or processor-specific
}

/// The ABI the ELF targets. Taken from the list on Wikipedia:
/// https://en.wikipedia.org/wiki/Executable_and_Linkable_Format#File_header
#[repr(u8)]
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum ABI {
	SystemV = 0,
	HPUX = 1,
	NetBSD = 2,
	Linux = 3,
	Hurd = 4,
	Solaris = 6,
	AIX = 7,
	IRIX = 8,
	FreeBSD = 9,
	Tru64 = 10,
	NovellModesto = 11,
	OpenBSD = 12,
	OpenVMS = 13,
	NonStopKernel = 14,
	AROS = 15,
	FenixOS = 16,
	NuxiCloud = 17,
	OpenVOS = 18,
}
