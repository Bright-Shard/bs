//! Defines structures for the RSDP and XSDP. They point to the Root System
//! Descriptor table and eXtended System Descriptor table, respectively.
//!
//! See `rsdt.rs` for info on those two tables.
//!
//! Resources:
//! - https://wiki.osdev.org/RSDP

use core::mem;

/// The "Root System Description Pointer".
#[repr(packed)]
pub struct Rsdp {
	/// The magic bytes for this struct. Should match [`Rsdp::SIGNATURE`].
	pub signature: [u8; 8],
	/// A way to validate this structure. The checksum, when added
	/// to every other field in this struct, should equal 0.
	pub checksum: u8,
	/// Identifies the OEM.
	pub oem_id: [u8; 6],
	/// The version of this structure. 0 for ACPI 1, 2 for ACPI 2 or later.
	pub revision: u8,
	/// Location of the Root System Descriptor. Only used for ACPI 1.
	pub rsdt_address: u32,
}
impl Rsdp {
	/// What the [`Rsdp.signature`] field should be set to.
	// `try_into` isn't const so we gotta do this to go string -> non-slice bytes
	pub const SIGNATURE: [u8; 8] = unsafe { *"RSD PTR ".as_ptr().cast() };

	/// Takes a raw pointer to an [`Rsdp`], and verifies it's a valid RSDP.
	///
	/// # Safety
	/// - `ptr` must be a non-null, aligned pointer
	/// - `ptr` must live for at least `'a`
	pub unsafe fn try_from_raw<'a>(ptr: *const Self) -> Result<&'a Self, RsdpXsdpError> {
		let rsdp = unsafe { &*ptr };

		if rsdp.signature != Self::SIGNATURE {
			return Err(RsdpXsdpError::Signature);
		}

		let mut checksum: u8 = 0;
		let bytes: &[u8; mem::size_of::<Rsdp>()] = unsafe { &*ptr.cast() };
		for byte in bytes {
			checksum = checksum.wrapping_add(*byte);
		}
		if checksum != 0 {
			return Err(RsdpXsdpError::Checksum);
		}

		Ok(rsdp)
	}
}

/// The "eXtended System Description Pointer". This is used instead of the
/// RSDP on systems with ACPI 2 or newer.
#[repr(packed)]
pub struct Xsdp {
	/// The XSDP has all the fields of the RSDP; it just adds more at the end.
	pub rsdp: Rsdp,
	/// The size of the XSDP.
	pub len: u32,
	/// The location of the Extended System Descriptor.
	pub xsd_address: u64,
	/// The checksum of the extended fields; does not include fields in the RSDP.
	pub extended_checksum: u8,
	pub reserved: [u8; 3],
}
impl Xsdp {
	/// Takes a raw pointer to an [`Xsdp`], and verifies it's a valid XSDP.
	///
	/// # Safety
	/// - `ptr` must be a non-null, aligned pointer
	/// - `ptr` must live for at least `'a`
	pub unsafe fn try_from_raw<'a>(ptr: *const Self) -> Result<&'a Self, RsdpXsdpError> {
		let rsdp = Rsdp::try_from_raw(ptr.cast())?;
		rsdp.try_into()
	}
}
impl<'a> TryFrom<&'a Rsdp> for &'a Xsdp {
	type Error = RsdpXsdpError;

	/// Converts an RSDP to an XSDP. An XSDP is a backwards-compatible RSDP present
	/// on ACPI v2 or newer. It points to an extended system descriptor instead of a
	/// root system descriptor.
	fn try_from(rsdp: &'a Rsdp) -> Result<Self, Self::Error> {
		if rsdp.revision != 2 {
			return Err(RsdpXsdpError::Revision(rsdp.revision));
		}

		let xsdp: &'a Xsdp = unsafe { mem::transmute(rsdp) };
		if xsdp.len as usize != mem::size_of::<Xsdp>() {
			return Err(RsdpXsdpError::Length);
		}

		let mut checksum: u8 = 0;
		let bytes: &[u8; mem::size_of::<Xsdp>()] = unsafe { mem::transmute(xsdp) };
		for byte in &bytes[mem::size_of::<Rsdp>() + 1..] {
			checksum = checksum.wrapping_add(*byte);
		}
		if checksum != 0 {
			return Err(RsdpXsdpError::Checksum);
		}

		Ok(xsdp)
	}
}

#[derive(Debug)]
/// An error while verifying an [`Rsdp`] or an [`Xsdp`].
pub enum RsdpXsdpError {
	/// The signature wasn't `RSD PTR `.
	Signature,
	/// BS only supports revision 2 XSDPs. These should be present on ACPI 2+ systems.
	Revision(u8),
	/// Checksum verification failed.
	Checksum,
	/// Extended checksum verification failed.
	ExtendedChecksum,
	/// The XSDP's length didn't match the size of [`Xsdp`].
	Length,
}
