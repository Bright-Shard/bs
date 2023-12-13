//! The Global Descriptor Table, or GDT, is a table that defines memory for the CPU with memory segments. Memory segments
//! have a base address, size, permissions, and a few extra flags. Segmented memory is a deprecated model - now replaced
//! with memory paging (see paging.rs) - but is still required to put the CPU into protected (32-bit) or extended (64-bit)
//! mode.
//!
//! The GDT is just an array of 8-byte segment descriptors. Each segment descriptor defines a segment of memory and its permissions.
//! Segments are allowed to overlap with each other; the simplest GDT configuration is just defining all memory as RWX by
//! defining a read/write segment that spans all memory and an executable segment that spans all memory (this setup is *quite*
//! insecure and should not be used without other memory protections).
//!
//! The GDT is not stored directly in x86. Instead, the GDTR register stores a *GDT Descriptor*, which stores the size and
//! location of the GDT.
//!
//! Resources:
//! - https://wiki.osdev.org/Global_Descriptor_Table
//! - https://wiki.osdev.org/GDT_Tutorial
//! - https://www.cs.bham.ac.uk/~exr/lectures/opsys/10_11/lectures/os-dev.pdf (the "Entering 32-bit Protected Mode" chapter)

/// The "limit" value of a segment descriptor is actually a u20, but Rust is sane and doesn't have a data type for it, so
/// instead it's stored as a u32 and then compared with this to make sure it's a valid u20 as well.
pub const U20_MAX: u32 = 0b0000_0000_0000_1111_1111_1111_1111_1111;

/// The literal, in-memory representation of a segment descriptor is just 8 bytes.
pub type SegmentDescriptor = [u8; 8];

/// The GDT is made up of Segment Descriptors, 8-byte structures that describe & configure a segment of memory.
/// The values in a descriptor aren't continuous - some numbers are defined in multiple bytes, but their bits are
/// scattered across the descriptor (check the OSDev wiki for more info). Thus, this struct is only used to build
/// a descriptor, and does not represent its actual layout in memory. The `build` method converts it to actual binary.
pub struct SegmentDescriptorBuilder {
    /// The minimum address for this region of memory.
    pub base: u32,
    /// The maximum address for this region of memory. Can be in bytes or 4kib pages, depending on a flag in
    /// the descriptor's flags. This is actually a 20-bit value, but Rust is sane and does not have a u20,
    /// so we use a u32 here and error if the value is too big.
    pub limit: u32,
    /// Flags for this segment.
    pub flags: SegmentFlagsBuilder,
    /// Permissions for this segment.
    pub access: SegmentAccessBuilder,
}
impl SegmentDescriptorBuilder {
    /// Converts the segment descriptor into its actual, 8-byte form. The layout of this structure is so
    /// confusing and twisted that I won't even bother explaining it here; check the OSDev wiki page instead:
    /// https://wiki.osdev.org/Global_Descriptor_Table#Segment_Descriptor
    pub const fn build(self) -> SegmentDescriptor {
        if self.limit > U20_MAX {
            panic!("A memory segment's limit must fit in a u20");
        }

        let limit = self.limit >> 4;
        let limit = limit.to_ne_bytes();
        let base = self.base.to_ne_bytes();
        [
            base[0],
            self.flags.build() | limit[0],
            self.access.build(),
            base[1],
            base[2],
            base[3],
            limit[1],
            limit[2],
        ]
    }
}

/// The segment's access byte controls permissions for this memory segment.
pub struct SegmentAccessBuilder {
    /// If this segment is in-memory.
    pub present: bool,
    /// The privilege of this segment, where 0 is highest/kernel privilege and 3 is the lowest/user privilege.
    /// This should technically be a u2, but once again Rust is sane and doesn't have such a bizarre type, so we
    /// just error if the value is greater than 3.
    pub privilege: u8,
    /// If this segment is a system segment. I'm not entirely sure what this means, but it appears to be used
    /// when hardware task switching is involved.
    pub non_system: bool,
    /// When true, this segment is executable (eg, code). When false, this segment is a data segment and cannot
    /// be executed.
    pub executable: bool,
    /// This has different meanings for data segments and code segments:
    ///
    /// **Code Segments**: If this data is "conforming". Conforming code can be executed from other code with a
    /// lower ring/privelege level. Non-conforming code can only be executed from other code with an equal ring
    /// level.
    ///
    /// **Data Segments**: The direction of the data. When false, code grows up (limit > base). When true, the
    /// code grows down (base > limit).
    pub direction_conforming: bool,
    /// This grants extra read/write permissions to this segment. Code segments are always executable, never writeable,
    /// and will be readable if this value is true. Data segments are always readable, never executable, and will be
    /// writeable if this segment is true.
    pub read_write: bool,
    /// The CPU sets this flag to true when it accesses this segment for the first time, if it's not already true.
    /// If the GDT is stored in non-writeable memory, the CPU may trigger a page fault when accessing this segment,
    /// because it'll try to set this flag to true but the GDT will be read-only.
    pub accessed: bool,
}
impl SegmentAccessBuilder {
    /// Converts the access values into a byte. The layout is `VPPS_EDRA`, where V is if the segment is valid, P is its
    /// privilege, S is if it's a system segment, E is if it's executable, D is its direction or if it's conforming, R is
    /// if it has an extra read/write permissions, and A is if it's been accessed.
    pub const fn build(self) -> u8 {
        let mut result = 0;

        if self.present {
            result |= 0b1000_0000;
        }

        match self.privilege {
            0 => {}
            1 => result |= 0b0010_0000,
            2 => result |= 0b0100_0000,
            3 => result |= 0b0110_0000,
            _ => panic!("A memory segment's privilege can only be between 0 and 3"),
        }

        if self.non_system {
            result |= 0b0001_0000;
        }
        if self.executable {
            result |= 0b0000_1000;
        }
        if self.direction_conforming {
            result |= 0b0000_0100;
        }
        if self.read_write {
            result |= 0b0000_0010;
        }
        if self.accessed {
            result |= 0b0000_0001;
        }

        result
    }
}

/// The segment's flags configure if the segment limit is in bytes or pages and the segment's bitness.
pub struct SegmentFlagsBuilder {
    /// When true, the segment limit is evaluated in 4kib pages. When false, it's evaluated in bytes.
    pub paged_limit: bool,
    /// When true, this segment is for 32-bit protected mode memory. When false, it's for 16-bit real mode memory.
    pub protected: bool,
    /// When true, this segment is for 64-bit memory. When false, it's for 32-bit or 16-bit memory, depending on
    /// the protected field above. When this is true, protected should always be false.
    pub long: bool,
}
impl SegmentFlagsBuilder {
    /// Converts the flags into 4 bits. The more significant 4 bits are the flags - the less significant 4 bits will
    /// end up being part of the limit value. The layout is `0SPL`, where S is if the segment is paged, P is if it's
    /// in protected/32-bit mode, and L is if it's in long/64-bit mode. The first bit is unused.
    pub const fn build(self) -> u8 {
        let mut result = 0;

        if self.paged_limit {
            result |= 0b1000_0000;
        }
        if self.protected {
            result |= 0b0100_0000;
        }
        if self.long {
            if self.protected {
                panic!("`protected` flag must be false for 64-bit segments");
            }

            result |= 0b0010_0000;
        }

        result
    }
}

/// Metadata about the GDT. This struct is what is actually stored in x86, instead of the GDT being stored directly.
#[repr(packed)]
pub struct GDTDescriptor {
    /// The address of the GDT. This is a u32 on 32-bit systems and a u64 on 64-bit systems.
    pub offset: u64,
    /// The size of the GDT in bytes, minus 1. The subtraction occurs because the max value of a u16 is 1 less than
    /// the maximum possible size of the GDT. I think this happens because the GDT always has to have at least 1 value,
    /// a null segment, but u16s start at 0.
    pub size: u16,
}
