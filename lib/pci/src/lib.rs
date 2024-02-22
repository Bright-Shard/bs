#![no_std]

pub mod address_space;
pub mod classification;

use {address_space::*, classification::*};

/// A wrapper around [`PciDeviceAddress`] and the classification types in [`classification`] that
/// makes it easy to read a PCI device's configuration.
pub struct PciDevice {
	/// Used to access the PCI device's address space.
	address: PciDeviceAddress,
	/// Caches values from the PCI configuration space. There are 256 bytes in the configuration
	/// space. Only 32 bits can be read at a time, so it's split into 64 4-byte registers.
	cache: [Option<[u8; 4]>; 64],
}
impl PciDevice {
	/// Attempts to access a PCI function on a PCI device on a PCI bus. Will return `None` if no device
	/// exists at that bus/device/function.
	pub fn new(bus: u8, device: u8, function: u8) -> Option<Self> {
		let address = PciDeviceAddress::new()
			.with_bus(bus)
			.with_device(device)
			.with_function(function);

		let mut this = Self {
			address,
			cache: [None; 64],
		};

		// If the device isn't present, a PCI read will return `0xFFFFFFFF`. That's an invalid vendor
		// so it immediately means the device is not present.
		// read_register also currently returns `None` on `0xFFFFFFFF`.
		this.read_register(0)?;

		Some(this)
	}

	/// Attempts to identify the PCI device's vendor. Returns `None` if the vendor is unknown,
	/// which will happen if the vendor isn't in BS' vendor enum (ie BS' vendor list is out of date
	/// or incomplete).
	pub fn vendor(&mut self) -> Option<Vendor> {
		let bytes = self.read_register(0)?;
		let vendor_id = u16::from_le_bytes([bytes[1], bytes[0]]);

		vendor_id.try_into().ok()
	}
	/// Attempts to identify the PCI device's class and subclass. This uses the PCI class list from
	/// the OSDev wiki, which *should* be complete and list every class; just in case it doesn't, though,
	/// this will return `None` for an unrecognised class.
	pub fn class(&mut self) -> Option<Class> {
		let bytes = self.read_register(2)?;

		Class::from_bytes(bytes[3], bytes[2])
	}
	/// Gets the header metadata from the configuration space. See [`HeaderMeta`].
	pub fn header(&mut self) -> Option<HeaderMeta> {
		let bytes = self.read_register(3)?;

		bytes[2].try_into().ok()
	}
	/// Get the `prog_if` byte.
	pub fn programming_interface(&mut self) -> Option<u8> {
		let bytes = self.read_register(2)?;

		Some(bytes[1])
	}

	/// Read a specific register from the PCI configuration space. This will get the value from the cache
	/// if it exists; otherwise it will get the value from PCI and store the result in cache. Returns `None`
	/// if the value is `0xFFFFFFFF`.
	pub fn read_register(&mut self, register: u8) -> Option<[u8; 4]> {
		match self.cache[register as usize] {
			Some(val) => Some(val),
			None => {
				let val = self.read_register_uncached(register)?;
				self.cache[register as usize] = Some(val);
				Some(val)
			}
		}
	}
	/// Read a register from the PCI configuration space. This will always read from PCI, and never
	/// reads from or writes to the cache. Returns `None` if the value is `0xFFFFFFFF`.
	pub fn read_register_uncached(&self, register: u8) -> Option<[u8; 4]> {
		match self.address.clone().with_register(register).read() {
			0xFFFFFFFF => None,
			val => Some(val.to_ne_bytes()),
		}
	}

	/// Get the PCI bus this device is on.
	#[inline(always)]
	pub fn bus(&self) -> u8 {
		self.address.bus()
	}
	/// Get the ID of this device on its PCI bus.
	#[inline(always)]
	pub fn device(&self) -> u8 {
		self.address.device()
	}
	/// Get the ID of this function on its device in the PCI bus.
	///
	/// A PCI device may have multiple functions. Each function technically
	/// resides on the same device, but gets its own PCI configuration space,
	/// so it's easiest to just treat each function as a separate device.
	#[inline(always)]
	pub fn function(&self) -> u8 {
		self.address.function()
	}
}
