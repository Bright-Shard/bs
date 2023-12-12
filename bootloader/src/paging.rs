//! Enables paging in memory, which defines available memory and its permissions. This is required for entering 64-bit mode.

pub struct PageTableBuilder {
    /// If the page is actually in-memory. Could be false if, for example, the page has been swapped to disk.
    present: bool,
    /// When true, the page is read-write. When false, it's read-only.
    mutable: bool,
    /// When true, the page is accessable to all. When false, only privileged code can access it.
    unprivileged: bool,
    /// Enables write-through caching. When false, it uses write-back caching instead.
    write_through: bool,
    /// When true, this page will not be cached.
    cache_disabled: bool,
    /// If this entry was read during virtual address translation.
    accessed: bool,
    /// If this page has been written to.
    dirty: bool,
    page_attribute_table: bool,
    global: bool,
    /// The address of this entry - either an actual memory address for page tables, or the address of
    /// a sub-table for nested tables.
    address: u32,
}
