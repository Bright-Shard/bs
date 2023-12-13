//! Memory pages are how memory permissions are set for the CPU. They supercede the older, segmented memory model.
//! The pages are configured via 4 nested tables: The page map level 4, the page directory pointer table,
//! the page directory, and the page table. Those names are definitely confusing, but you can kinda remember it as
//! "the longer the name, the higher up the table". The page map level 4 stores entries that point to page directory
//! pointer tables. Page directory pointer tables store entries that point to page directories. Page directories
//! store entries that point to page tables. Page tables store entries that configure 4kib of memory each, unless
//! huge pages are enabled, in which case they configure 4mib of memory each. Each table stores 512 entries in
//! 64-bit mode, or 1024 entries in 32-bit mode. This means they take up the same amount of memory in either mode.
//! Also, in 32-bit mode, the page map level 4 and page directory pointer tables do not exist; only the page tables
//! and a page directory table are used.
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
//! Virtual addresses are converted to physical addresses by using them to index into page maps. The first 10 bits, for example,
//! can index into the top-level page map; the next 10 bits into the table after that; and so on until the last 10 bits are reached,
//! which act as an offset from the page's base address.
//!
//! Resources:
//! - https://wiki.osdev.org/Paging
//! - https://wiki.osdev.org/Setting_Up_Long_Mode#Setting_up_the_Paging
//! - https://wiki.osdev.org/Entering_Long_Mode_Directly

/// Builds an entry for any of the types of page tables/directories/etc. They all share this same layout, the only difference being
/// what the address field points to (a sub-table for all the tables except for page tables, where it points to actual memory).
pub struct PageBuilder {
    /// If the page is actually in-memory. Could be false if, for example, the page has been swapped to disk.
    pub present: bool,
    /// When true, the page is read-write. When false, it's read-only.
    pub mutable: bool,
    /// When true, the page is accessable to all. When false, only privileged code can access it.
    pub unprivileged: bool,
    /// Enables write-through caching. When false, it uses write-back caching instead.
    pub write_through: bool,
    /// When true, this page will not be cached.
    pub cache_disabled: bool,
    /// If this entry was read during virtual address translation.
    pub accessed: bool,
    /// If this page has been written to.
    pub dirty: bool,
    /// Unsure what this does, but BS doesn't use page attributes so it's always false.
    pub page_attribute_table: bool,
    /// Also unsure what this does.
    pub global: bool,
    /// The address of this entry - either an actual memory address for page tables, or the address of
    /// a sub-table for nested tables. This is a u20 on 32-bit systems
    pub address: u64,
}
