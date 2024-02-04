//! Defines the Root System Descriptor Table (RSDT) and eXtended System Descriptor Table
//! (XSDT), which point to other tables in memory that give information about the hardware
//! on the computer.
//!
//! Sources:
//! - https://wiki.osdev.org/RSDT
//! - https://wiki.osdev.org/XSDT

use core::{mem, slice};

/// The SDT/System Descriptor Table. Essentially used as a basis
/// for all the other tables here.
#[repr(packed)]
pub struct SystemDescriptor {
	/// An identifier for this table.
	pub signature: [u8; 4],
	/// The size of the entire table, including the system descriptor and its data.
	pub len: u32,
	/// The version of this table.
	pub revision: u8,
	/// Used to verify the contents of the table. All of the bytes in the table, including
	/// this byte, should add up to 0.
	pub checksum: u8,
	/// An identifier for the manufacturer.
	pub oem_id: [u8; 6],
	pub oem_table_id: [u8; 8],
	pub oem_revision: u32,
	pub creator_id: u32,
	pub creator_revision: u32,
}
impl SystemDescriptor {
	/// Takes a possible pointer to an SDT and ensures it's a valid [`SystemDescriptor`].
	///
	/// # Safety
	/// - `ptr` must be a non-null, aligned pointer
	/// - `ptr` must live for at least `'a`
	pub unsafe fn try_from_raw<'a>(ptr: *const Self) -> Result<&'a Self, SystemDescriptorError> {
		let descriptor = unsafe { &*ptr };

		if descriptor.len < mem::size_of::<SystemDescriptor>() as u32 {
			return Err(SystemDescriptorError::Length);
		}
		let bytes = unsafe { slice::from_raw_parts(ptr.cast::<u8>(), descriptor.len as _) };
		let mut checksum: u8 = 0;
		for byte in bytes {
			checksum = checksum.wrapping_add(*byte);
		}
		if checksum != 0 {
			return Err(SystemDescriptorError::Checksum);
		}

		Ok(descriptor)
	}
}

/// Errors while verifying a [`SystemDescriptor`].
#[derive(Debug)]
pub enum SystemDescriptorError {
	/// The bytes of the descriptor added together didn't equal 0.
	Checksum,
	/// The length field of the descriptor was less than the size of a descriptor.
	Length,
}

/// Abstracts over number types that can be converted to pointers.
pub trait ToPtr {
	fn to_ptr<T>(&self) -> *const T;
}
impl ToPtr for u64 {
	#[inline(always)]
	fn to_ptr<T>(&self) -> *const T {
		(*self) as _
	}
}
impl ToPtr for u32 {
	#[inline(always)]
	fn to_ptr<T>(&self) -> *const T {
		(*self) as _
	}
}

/// An abstraction over [`Rsdt`] and [`Xsdt`], which are identical except for their pointer sizes.
#[repr(packed)]
pub struct Sdt<'a, PtrSize: ToPtr> {
	pub descriptor: &'a SystemDescriptor,
	/// Pointers to other system tables.
	pub tables: &'a [PtrSize],
}
impl<'a, PtrSize: ToPtr> Sdt<'a, PtrSize> {
	/// Takes a possible pointer to an RSDT/XSDT and ensures it's a valid [`Rsdt`]/[`Xsdt`].
	///
	/// # Safety
	/// - `ptr` must be a non-null, aligned pointer
	/// - `ptr` must live for at least `'a`
	pub unsafe fn try_from_raw(ptr: *const Self) -> Result<Self, SystemDescriptorError> {
		let descriptor = SystemDescriptor::try_from_raw(ptr.cast())?;

		let tables_addr = (ptr as *const () as usize) + mem::size_of::<SystemDescriptor>();
		let tables_len = descriptor.len as usize - mem::size_of::<SystemDescriptor>();
		let num_entries = tables_len / mem::size_of::<PtrSize>();
		let tables = core::slice::from_raw_parts(tables_addr as *const PtrSize, num_entries);

		Ok(Self { descriptor, tables })
	}

	/// Find a table pointed to by this [`Rsdt`]/[`Xsdt`]. Both tables store a list of pointers
	/// that point to other tables. Those tables all start with a [`SystemDescriptor`], and can be
	/// identified by their 4-byte signature.
	pub fn find_table(&self, name: &str) -> Option<&SystemDescriptor> {
		let name = name.as_bytes();
		if name.len() != 4 {
			return None;
		}

		for table in self.tables.iter() {
			let descriptor = unsafe { SystemDescriptor::try_from_raw(table.to_ptr()) };
			if let Ok(descriptor) = descriptor {
				if descriptor.signature == name {
					return Some(descriptor);
				}
			}
		}

		None
	}
}

/// The Root System Descriptor Table. Stores pointers to other important tables in the system.
pub type Rsdt<'a> = Sdt<'a, u32>;
/// The eXtended System Descriptor Table. Stores pointers to other important tables in the system.
/// Identical to an [`Rsdt`] except it has 64-bit pointers, while an [`Rsdt`] has 32-bit pointers.
pub type Xsdt<'a> = Sdt<'a, u64>;
