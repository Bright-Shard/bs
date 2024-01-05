//! The BS bootloader loads whatever ELF file comes immediately after it on the disk.
//! This module loads that file into memory.
//!
//! Resources:
//! - https://en.wikipedia.org/wiki/Executable_and_Linkable_Format
//! - https://wiki.osdev.org/ELF
//! - http://www.skyfree.org/linux/references/ELF_Format.pdf
//! - https://www.man7.org/linux/man-pages/man5/elf.5.html

pub mod structs;

use {crate::*, core::slice, structs::*};

/// When the bootstrapper loads the bootloader, it passes it the last disk sector it read from + 1. When BS
/// is built, the bootstrapper, bootloader, and kernel are laid out sequentially on the disk, in that order.
/// This means that the sector passed from the bootstrapper is actually the first sector of the kernel.
pub fn parse_from_sector(sector: u8) {
    let mut sector = (sector - 1) as u64;
    let base_address = 0xFFFFF;
    let mut address = base_address;
    disks::read_sectors(&mut sector, 1, &mut address);

    let header = unsafe { &(address as *const FileHeader).read() };

    // Check that the file is a supported ELF file
    if header.magic_bytes != [0x7F, 0x45, 0x4C, 0x46] {
        panic!("Couldn't find the ELF magic bytes.",)
    }
    if header.bitness != Bitness::X64 {
        panic!("BS only supports 64-bit ELFs.")
    }
    if header.endianess != Endianess::NATIVE {
        todo!("Figure out non-native endianness")
    }
    if header.elf_version != 1 || header.header_version != 1 {
        panic!("BS only supports v1 ELFs.",)
    }
    if header.abi != ABI::SystemV {
        panic!("BS only supports the SystemV ABI.",)
    }
    let object_type = header.object_type;
    if object_type != ObjectType::Dyn {
        panic!("BS only supports position-independent ELF objects.",)
    }

    // Load the section header table
    let section_table_base = header.section_table_offset as u16;
    let section_table_len = header.section_header_size * header.section_table_entries;
    let section_table_end = section_table_base + section_table_len;
    if section_table_end > 512 {
        let sectors_to_read = section_table_end.div_ceil(512);
        disks::read_sectors(&mut sector, sectors_to_read as _, &mut address);
    }
    let section_header_table = unsafe {
        slice::from_raw_parts(
            (base_address + section_table_base) as *const SectionHeader,
            section_table_len as usize,
        )
    };

    // Find the file size from the ELF sections
    let mut total_size = 0;
    for section_header in section_header_table {
        total_size += section_header.size;
    }
    println!("ELF file size: {total_size}");
    let sectors_to_read = total_size.div_ceil(512);
    disks::read_sectors(&mut sector, sectors_to_read as _, &mut address);
}
