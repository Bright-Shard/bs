#![no_std]

pub mod classification;
pub mod configuration;

use {classification::*, configuration::*, core::ops::Deref};

/// A wrapper around [`PciDeviceAddress`] and the classification types in [`classification`] that
/// makes it easy to read a PCI device's configuration.
pub struct PciDevice {
	/// Used to access the PCI device's address space.
	address: PciDeviceAddress,
	/// Functions available on this device.
	functions: [Option<PciDeviceFunction>; 8],
	/// The function we're currently working with
	active_function: usize,
}
impl PciDevice {
	/// Attempts to access a PCI device on a PCI bus. Will return `None` if no device exists at
	/// the specified device ID and PCI bus or the device's class is unknown.
	pub fn new(bus: u8, device: u8) -> Option<Self> {
		let address = PciDeviceAddress::new().with_bus(bus).with_device(device);

		let mut this = Self {
			address,
			functions: [None, None, None, None, None, None, None, None],
			active_function: 0,
		};
		this.set_function(0).ok()?;

		Some(this)
	}

	/// Switch the actively controlled function on the device. Errors if that function doesn't exist.
	#[allow(clippy::result_unit_err)]
	pub fn set_function(&mut self, function_id: usize) -> Result<(), ()> {
		if function_id <= 7 {
			let function_cache = self.functions.get_mut(function_id).unwrap();
			if function_cache.is_none() {
				if let Some(function) =
					PciDeviceFunction::new(self.address.with_function(function_id as u8))
				{
					*function_cache = Some(function);
					self.active_function = function_id;
					return Ok(());
				} else {
					return Err(());
				}
			}

			self.active_function = function_id;
			Ok(())
		} else {
			Err(())
		}
	}

	#[inline(always)]
	pub fn bus(&self) -> u8 {
		self.address.bus()
	}
	#[inline(always)]
	pub fn device(&self) -> u8 {
		self.address.device()
	}
	#[inline(always)]
	pub fn function(&self) -> u8 {
		self.active_function as _
	}
}
impl Deref for PciDevice {
	type Target = PciDeviceFunction;

	fn deref(&self) -> &Self::Target {
		self.functions[self.active_function].as_ref().unwrap()
	}
}

/// Represents a single function on a PCI device.
pub struct PciDeviceFunction {
	pub class: Class,
	pub header_meta: HeaderMeta,
	pub programming_interface: u8,
	address: PciDeviceAddress,
}
impl PciDeviceFunction {
	pub fn new(address: PciDeviceAddress) -> Option<Self> {
		// If vendor == 0xFFFF, no device exists at that address.
		let vendor_bytes = address.with_register(0).read().to_ne_bytes();
		let vendor_id = u16::from_le_bytes([vendor_bytes[1], vendor_bytes[0]]);
		let vendor = vendor_id.try_into();
		if vendor == Ok(Vendor::Invalid) {
			return None;
		}

		let class_bytes = address.with_register(2).read().to_ne_bytes();
		let class_byte = class_bytes[3];
		let subclass = class_bytes[2];
		let class = Class::from_bytes(class_byte, subclass)?;
		let programming_interface = class_bytes[1];

		let header_bytes = address.with_register(3).read().to_ne_bytes();
		let header_byte = header_bytes[2];
		let header_meta = HeaderMeta::try_from(header_byte).ok()?;

		Some(Self {
			class,
			header_meta,
			programming_interface,
			address,
		})
	}

	/// Reads a 32-bit register from the PCI device's configuration space.
	pub fn read_register(&self, register: u8) -> u32 {
		self.address.with_register(register * 4).read()
	}
}
