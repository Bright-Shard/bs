//! Reads from and writes to PCI devices via the PCI configuration space.

use core::arch::asm;

/// Specifies an address in a PCI device's configuration space to be read.
///
/// This gets written to CPU I/O port `0xCF8`. It specifies a device on
/// a PCI bus, a PCI bus, and the part of that device's PCI configuration space
/// to read (only 32 bits can be read at a time). That part of the
/// configuration space is then read from CPU I/O port `0xCFC`.
#[repr(transparent)]
pub struct PciDeviceAddress(u32);
impl PciDeviceAddress {
	#[inline(always)]
	pub fn new() -> Self {
		Self::default()
	}

	/// Specifies the PCI bus to find this PCI device on.
	pub fn with_bus(&self, bus: u8) -> Self {
		let bus = (bus as u32) << 16;
		Self(self.0 | bus)
	}
	/// Specifies the ID of this PCI device on its bus.
	pub fn with_device(&self, device: u8) -> Self {
		let device = (device as u32) << 11;
		Self(self.0 | device)
	}
	/// Some PCI devices have multiple functions that all have separate PCI
	/// configurations. This selects a function on a device.
	pub fn with_function(&self, function: u8) -> Self {
		let function = (function as u32) << 8;
		Self(self.0 | function)
	}
	/// The PCI configuration space is 256 bytes in length, but only 32 bits
	/// can be read at a time. Each set of 32 bits is a register. This selects
	/// a register from the configuration space to read.
	pub fn with_register(&self, register: u8) -> Self {
		Self(self.0 | ((register as u32) * 4))
	}

	pub fn bus(&self) -> u8 {
		(self.0 >> 16) as u8
	}
	pub fn device(&self) -> u8 {
		(self.0 >> 11) as u8
	}
	pub fn function(&self) -> u8 {
		(self.0 >> 8) as u8
	}
	pub fn offset(&self) -> u8 {
		self.0 as u8
	}

	/// Writes this address to I/O port `0xCF8` and then reads the PCI
	/// configuration from I/O port `0xCFC`.
	pub fn read(&self) -> u32 {
		let mut result = self.0;
		unsafe {
			asm!(
				"push dx",

				"mov dx, 0xCF8",
				"out dx, eax",
				"mov dx, 0xCFC",
				"in eax, dx",

				"pop dx",
				// inout reads `result` into eax at the start
				// of the assembly and then reads eax to `result`
				// at the end of the assembly.
				inout("eax") result,
			)
		}

		// Make sure we read the output as little-endian
		// PCI is always little-endian
		u32::from_le_bytes(result.to_ne_bytes())
	}
}
impl Default for PciDeviceAddress {
	fn default() -> Self {
		// it starts enabled cause it's annoying to make another method for that
		Self(1u32 << 31)
	}
}
