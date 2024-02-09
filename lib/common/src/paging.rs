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
//! Note: All of the documentation here only applies to 64-bit mode, 32-bit mode has different tables, table sizes, etc.
//!
//! Resources:
//! - https://wiki.osdev.org/Paging
//! - https://wiki.osdev.org/Setting_Up_Long_Mode#Setting_up_the_Paging
//! - https://wiki.osdev.org/Entering_Long_Mode_Directly
//! - https://www.intel.com/content/www/us/en/developer/articles/technical/intel-sdm.html (specifically vol 3, chap 4)

use core::ops::{Deref, DerefMut};

/// How all 64-bit page tables are laid out in memory - 512 entries, each one 8 bytes in length.
#[repr(align(0x1000))]
pub struct PageMap<E: PageMapEntry>([E; 512]);
impl<E: PageMapEntry> PageMap<E> {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn ptr(&self) -> *const () {
		(&self.0 as *const [E; 512]).cast()
	}
}
impl<E: PageMapEntry> Default for PageMap<E> {
	fn default() -> Self {
		Self([E::default(); 512])
	}
}
impl<E: PageMapEntry> Deref for PageMap<E> {
	type Target = [E; 512];

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
impl<E: PageMapEntry> DerefMut for PageMap<E> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

/// Marker type for all the types of page maps.
pub trait PageMapEntry: Default + Copy {}

/// Automates flipping bits based on a boolean. Sets the bit if the boolean is true.
macro_rules! bitbool {
	($name:ident, $pos:literal, $var:expr) => {
		if $name {
			$var |= 1 << $pos;
		} else {
			$var ^= 1 << $pos;
		}
	};
}
/// Automates flipping bits based on a boolean. Sets the bit if the boolean is false.
macro_rules! inverse_bitbool {
	($name:ident, $pos:literal, $var:expr) => {
		if !$name {
			$var |= 1 << $pos;
		} else {
			$var ^= 1 << $pos;
		}
	};
}

/// Implements properties page maps share.
macro_rules! page_map_type {
	($name:ident) => {
		#[derive(Clone, Copy)]
		#[repr(transparent)]
		pub struct $name(u64);

		impl $name {
			/// Creates a new, empty entry.
			pub fn new() -> Self {
				Self(0)
			}

			/// Marks this page as present in-memory.
			///
			/// Default value: False, this page is marked as missing.
			pub fn set_present(&mut self, present: bool) -> &mut Self {
				bitbool!(present, 0, self.0);

				self
			}

			/// Allows writing to memory in this page.
			///
			/// Default value: False, this page is read-only.
			pub fn set_writable(&mut self, writable: bool) -> &mut Self {
				bitbool!(writable, 1, self.0);

				self
			}

			/// Allows user-mode code to access this page.
			///
			/// Default value: False, only the supervisor can access this page.
			pub fn set_user_mode(&mut self, user_mode: bool) -> &mut Self {
				bitbool!(user_mode, 2, self.0);

				self
			}

			/// When enabled, uses write-through caching. When disabled,
			/// write-back caching is used instead.
			///
			/// Write-through caching: When this page is written to, both
			/// the CPU cache and main memory will be updated. This causes
			/// 2 writes every time memory is updated.
			///
			/// Write-back caching: When this page is written to, only the CPU
			/// cache is updated. Main memory is written to lazily. Can interfere
			/// with memory-mapped I/O.
			///
			/// Default value: False, write-back caching is used.
			pub fn set_write_through_cache(&mut self, write_through: bool) -> &mut Self {
				bitbool!(write_through, 3, self.0);

				self
			}

			/// When enabled, this page can be cached in the CPU. When disabled,
			/// this page cannot be cached.
			///
			/// Default value: True, caching is enabled.
			pub fn set_caching(&mut self, cache_enabled: bool) -> &mut Self {
				inverse_bitbool!(cache_enabled, 4, self.0);

				self
			}

			/// A flag set by the CPU when this page is read from memory. Note that
			/// the CPU never clears this flag, so the OS is responsible for that.
			///
			/// Default value: False, the page has not yet been read.
			pub fn set_accessed(&mut self, accessed: bool) -> &mut Self {
				bitbool!(accessed, 5, self.0);

				self
			}

			/// Allows data in this page to be executed as code. This bit is only used
			/// if the NXE bit is set in the EFER model-specific register. If the NXE
			/// bit is not set, this flag should not be set.
			///
			/// Default value: True, data in this page can be executed.
			pub fn set_executable(&mut self, executable: bool) -> &mut Self {
				inverse_bitbool!(executable, 63, self.0);

				self
			}

			/// Sets the address this entry points to.
			pub fn set_address(&mut self, address: u64) -> &mut Self {
				if (address % 4096) != 0 {
					panic!("Page table addresses must be 4kb-aligned");
				}
				self.0 |= address;

				self
			}
		}

		impl Default for $name {
			/// Creates a new, empty entry.
			fn default() -> Self {
				Self(0)
			}
		}

		impl PageMapEntry for $name {}
	};
}

page_map_type!(PageMapLevel4Entry);
page_map_type!(PageDirectoryPointerTableEntry);
page_map_type!(PageDirectoryEntry);
page_map_type!(PageTableEntry);

// TODO: There are more page attributes to support, but they aren't standard across all the page map types.
