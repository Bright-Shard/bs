//! Memory pages are how memory permissions are set for the CPU. They supercede the older, segmented memory model in the GDT.
//! The pages are configured via 4 nested tables: The page map level 4, the page directory pointer table,
//! the page directory, and the page table. Those names are definitely confusing, but you can kinda remember it as
//! "the longer the name, the higher up the table". The page map level 4 stores entries that point to page directory
//! pointer tables. Page directory pointer tables store entries that point to page directories. Page directories
//! store entries that point to page tables. Page tables store entries that configure 4kb of memory each. Each table stores
//! 512 entries.
//!
//! Some of the tables can also directly map larger sections of memory rather than point to their sub-tables. Page directory
//! pointer tables can directly map 1gb of memory instead of pointing to page directories, and page directories can directly
//! map 2mb of memory instead of pointing to page tables.
//!
//! The nested table design seems confusing, but gives a *huge* advantage in how much memory can be mapped. If
//! the CPU only stored a page table, it'd only be able to map 512 4kib pages of memory (or 1024 pages in 32-bit mode).
//! By using a page directory, it can store 512 tables of 512 4kib pages of memory. Adding the page directory pointer table
//! makes that 512 tables of 512 tables of 512 4kib pages of memory, and so on, and so forth. Adding more tables massively
//! increases how much memory the system can use - and, to this end, there's propositions for level 5 paging that introduces
//! a *fifth* table called the page map level 5 (at long last, consistent naming!). Some CPUs do support it, but level 4
//! paging already supports such a ridiculous amount of memory that there's no point in BS trying to utilise level 5 (it'd only
//! break compatibility with CPUs that don't have level 5 paging, or require more code to be compatible with them, with no
//! benefit since that memory won't even be used).
//!
//! Virtual addresses are converted to physical addresses by using them to index into page maps. The implementation depends
//! on the bitness and how many levels of paging there are, but the general idea is that several bits will index into the
//! top-level page map, then the next few will index into the page map under that, etc. The last few bits will be an offset
//! from the start of that page to the actual needed memory address.
//!
//! Note: All of the documentation here only applies to 64-bit mode, 32-bit mode has different tables, table sizes, etc. The
//! structures of 32-bit tables are also somewhat different from 64-bit ones.
//!
//! Resources:
//! - https://wiki.osdev.org/Paging
//! - https://wiki.osdev.org/Setting_Up_Long_Mode#Setting_up_the_Paging
//! - https://wiki.osdev.org/Entering_Long_Mode_Directly
//! - https://www.intel.com/content/www/us/en/developer/articles/technical/intel-sdm.html (specifically vol 3, chap 4)

#![allow(dead_code)]

use core::mem::ManuallyDrop;

const U4_MAX: u8 = 0b0000_1111;

/// How all 64-bit page tables are laid out in memory - 512 entries, each one 8 bytes in length.
#[repr(align(0x1000))]
pub struct PageMap(pub ManuallyDrop<[u64; 512]>);
impl PageMap {
    pub const fn new() -> Self {
        Self(ManuallyDrop::new([0; 512]))
    }
}

/// The Page Map Level 4 is the top-level page table. Its entries store pointers to Page Directory Pointer Tables.
pub struct PageMapLevel4EntryBuilder {
    /// When true, the page is actually loaded in-memory. This may be false if the page is unloaded or swapped to disk.
    pub present: bool,
    /// When true, the page is read-write. When false, the page is read-only.
    pub writable: bool,
    /// When true, data in this page is available to everyone in user-mode. When false, it's only available to privileged
    /// code.
    pub user_mode: bool,
    /// Enables write-through caching. When false, write-back caching is used instead. Write-through caching updates the
    /// CPU cache *and* memory when data in this page is written to, essentially causing 2 writes every write. Write-back
    /// caching will only update the CPU cache on a write, and will update memory later as needed.
    pub write_through: bool,
    /// When true, this page will not be cached in the CPU.
    pub cache_disabled: bool,
    /// The CPU will set this to true on its own when a page is read.
    pub accessed: bool,
    /// The location of the Page Directory Pointer Table for this entry.
    pub address: u64,
    /// If the NXE bit is set: Prevents data in this page from being executed as code. Otherwise: Reserved, must be false.
    /// NXE is a bit in a control register that must be set separately.
    pub execute_disable: bool,
}
impl PageMapLevel4EntryBuilder {
    /// Creates an 8-byte Page Map Level 4 entry.
    pub const fn build(self) -> u64 {
        let mut result = 0;

        if self.present {
            result |= 1;
        }
        if self.writable {
            result |= 1 << 1;
        }
        if self.user_mode {
            result |= 1 << 2;
        }
        if self.write_through {
            result |= 1 << 3;
        }
        if self.cache_disabled {
            result |= 1 << 4;
        }
        if self.accessed {
            result |= 1 << 5;
        }
        if self.execute_disable {
            result |= 1 << 63;
        }

        #[cfg(debug_assertions)]
        if (self.address % 4096) != 0 {
            panic!("Page table addresses must be 4kb-aligned");
        }
        result |= self.address;

        result
    }
}

/// Page Directory Pointer Tables are the 2nd-level page tables, stored under the Page Map Level 4.
/// Their entries either map 1gb of memory or point to Page Directories.
pub struct PageDirectoryPointerTableEntryBuilder {
    /// When true, the page is actually loaded in-memory. This may be false if the page is unloaded or swapped to disk.
    pub present: bool,
    /// When true, the page is read-write. When false, the page is read-only.
    pub writable: bool,
    /// When true, data in this page is available to everyone in user-mode. When false, it's only available to privileged
    /// code.
    pub user_mode: bool,
    /// Enables write-through caching. When false, write-back caching is used instead. Write-through caching updates the
    /// CPU cache *and* memory when data in this page is written to, essentially causing 2 writes every write. Write-back
    /// caching will only update the CPU cache on a write, and will update memory later as needed.
    pub write_through: bool,
    /// When true, this page will not be cached in the CPU.
    pub cache_disabled: bool,
    /// The CPU will set this to true on its own when a page is read.
    pub accessed: bool,
    /// If this entry is a direct map, this indicates the page has been written to/changed. Otherwise, this is ignored.
    pub dirty: bool,
    /// When true, this entry directly maps 1gib of memory. When false, this entry points to a Page Directory.
    pub direct_map: bool,
    /// If this entry is a direct map, this makes it global. Otherwise, it's ignored.
    pub global: bool,
    /// If this entry is a direct map, this is used for configuring the PAT, which can give a type to this page of memory.
    /// Otherwise, this is reserved (must be false).
    pub pat: bool,
    /// If this entry is a direct map, the base of the gigabyte of memory being paged. Otherwise, this is a pointer
    /// to a Page Directory.
    pub address: u64,
    /// If this entry is a direct map, and protection keys are enabled, this can control access rights for this page. This
    /// is actually a u4 and must fit in 4 bits.
    pub protection_key: Option<u8>,
    /// If no-execute is enabled, this disables instruction fetches from this region of memory; otherwise, it's reserved
    /// and must be false.
    pub execute_disable: bool,
}
impl PageDirectoryPointerTableEntryBuilder {
    /// Builds the actual, 8-byte struct.
    pub const fn build(self) -> u64 {
        let mut result = 0;

        if self.present {
            result |= 1;
        }
        if self.writable {
            result |= 1 << 1;
        }
        if self.user_mode {
            result |= 1 << 2;
        }
        if self.write_through {
            result |= 1 << 3;
        }
        if self.cache_disabled {
            result |= 1 << 4;
        }
        if self.accessed {
            result |= 1 << 5;
        }
        if self.dirty {
            result |= 1 << 6;
        }
        if self.direct_map {
            result |= 1 << 7;
        }
        if self.global {
            result |= 1 << 8;
        }
        if self.pat {
            #[cfg(debug_assertions)]
            if !self.direct_map {
                panic!("Non-direct-mapping tables cannot set the PAT bit");
            }
            result |= 1 << 12;
        }
        if let Some(key) = self.protection_key {
            #[cfg(debug_assertions)]
            if key > U4_MAX {
                panic!("Protection key is too large, it must fit in 4 bits");
            }
            result |= (key as u64) << 62;
        }
        if self.execute_disable {
            result |= 1 << 63;
        }

        // TODO: Should probably check address alignment
        result |= self.address;

        result
    }
}

/// Page Directories are 3rd-level page tables, stored under Page Directory Pointer Tables. Their entries either map
/// 2mib of memory or point to Page Tables. They are nearly identical to Page Directory Pointer Tables, except a direct
/// map only maps 2mib of memory instead of 1gib.
pub type PageDirectoryEntryBuilder = PageDirectoryPointerTableEntryBuilder;

/// Page Tables are the lowest type of page table, stored under Page Directories. Their entries map 4kib of memory each.
pub struct PageTableEntryBuilder {
    /// When true, the page is actually loaded in-memory. This may be false if the page is unloaded or swapped to disk.
    pub present: bool,
    /// When true, the page is read-write. When false, the page is read-only.
    pub writable: bool,
    /// When true, data in this page is available to everyone in user-mode. When false, it's only available to privileged
    /// code.
    pub user_mode: bool,
    /// Enables write-through caching. When false, write-back caching is used instead. Write-through caching updates the
    /// CPU cache *and* memory when data in this page is written to, essentially causing 2 writes every write. Write-back
    /// caching will only update the CPU cache on a write, and will update memory later as needed.
    pub write_through: bool,
    /// When true, this page will not be cached in the CPU.
    pub cache_disabled: bool,
    /// The CPU will set this to true on its own when a page is read.
    pub accessed: bool,
    /// The CPU will set this to true on its own to indicate that the page has been written to/changed.
    pub dirty: bool,
    /// This is used for configuring the PAT, which can give a type to this page of memory.
    pub pat: bool,
    /// This makes the page global.
    pub global: bool,
    /// The base address of the 4kb of memory being mapped.
    pub address: u64,
    /// If this entry is a direct map, and protection keys are enabled, this can control access rights for this page. This
    /// is actually a u4 and must fit in 4 bits.
    pub protection_key: Option<u8>,
    /// If no-execute is enabled, this disables instruction fetches from this region of memory; otherwise, it's reserved
    /// and must be false.
    pub execute_disable: bool,
}
impl PageTableEntryBuilder {
    /// Builds the actual, 8-byte struct.
    pub const fn build(self) -> u64 {
        let mut result = 0;

        if self.present {
            result |= 1;
        }
        if self.writable {
            result |= 1 << 1;
        }
        if self.user_mode {
            result |= 1 << 2;
        }
        if self.write_through {
            result |= 1 << 3;
        }
        if self.cache_disabled {
            result |= 1 << 4;
        }
        if self.accessed {
            result |= 1 << 5;
        }
        if self.dirty {
            result |= 1 << 6;
        }
        if self.pat {
            result |= 1 << 7;
        }
        if self.global {
            result |= 1 << 8;
        }
        if let Some(key) = self.protection_key {
            #[cfg(debug_assertions)]
            if key > U4_MAX {
                panic!("Protection key is too large, it must fit in 4 bits");
            }
            result |= (key as u64) << 59;
        }
        if self.execute_disable {
            result |= 1 << 63;
        }

        result
    }
}
