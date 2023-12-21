//! The BS bootloader loads whatever ELF file comes immediately after it on the disk.
//! This module loads that file into memory.
//!
//! Resources:
//! - https://en.wikipedia.org/wiki/Executable_and_Linkable_Format
//! - https://wiki.osdev.org/ELF
//! - http://www.skyfree.org/linux/references/ELF_Format.pdf

use crate::*;

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
    pub header_table: u64,
    /// An offset to the program section table of this ELF file.
    pub section_table: u64,
    /// Architecture-specific flags.
    pub flags: u32,
    /// The size of this ELF header. It ironically comes after all the fields that change size,
    /// meaning its location in the structure changes... based on the structure's size.
    pub size: u16,
    /// The size of a program header entry.
    pub program_header_entry_size: u16,
    /// The number of entries in the program header.
    pub program_header_entries: u16,
    /// The size of a section header entry.
    pub section_header_entry_size: u16,
    /// The number of entries in the section header.
    pub section_header_entries: u16,
    /// The index into the section header that has section names.
    pub section_header_names_index: u16,
}

/// Each program header describes how parts of the file should be loaded in memory.
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

/// Each section header describes a part of the ELF file.
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

/// When the bootstrapper loads the bootloader, it passes it the last disk sector it read from + 1. When BS
/// is built, the bootstrapper, bootloader, and kernel are laid out sequentially on the disk, in that order.
/// This means that the sector passed from the bootstrapper is actually the first sector of the kernel.
///
/// This loads the kernel's ELF, starting at that sector, into memory. The bootloader is loaded at 0x4000
/// (memory below that is for the stack), and the kernel is loaded at 0x7c00. This means the entire bootloader
/// must fit into about 15kib of memory. The kernel is free to use the rest of the memory as it pleases.
pub fn parse_from_sector(sector: u8) {
    let sector = sector as u64;
    let address = 0x7C00;
    disks::read_sectors(sector - 1, 1, address);

    let elf_header = unsafe { &(address as *const FileHeader).read() };

    // Check that the file is a supported ELF file
    #[cfg(debug_assertions)]
    {
        if elf_header.magic_bytes != [0x7F, 0x45, 0x4C, 0x46] {
            panic!("Couldn't find the ELF magic bytes.",)
        }
        if elf_header.bitness != Bitness::X64 {
            panic!("BS only supports 64-bit ELFs.")
        }
        if elf_header.elf_version != 1 || elf_header.header_version != 1 {
            panic!("BS only supports v1 ELFs.",)
        }
        if elf_header.abi != ABI::SystemV {
            panic!("BS only supports the SystemV ABI.",)
        }
        let object_type = elf_header.object_type;
        if object_type != ObjectType::Dyn {
            panic!("BS only supports position-independent ELF objects.",)
        }
    }
}
