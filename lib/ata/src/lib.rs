#![no_std]

use {
	core::arch::asm,
	pci::{
		classification::{Class, MassStorageControllerSubclass},
		PciDevice,
	},
};

mod enums;
pub use enums::*;

/// Represents an IDE controller on the PCI bus. Each controller has two channels, which can each hold two drives.
pub struct IdeController {
	/// The first channel on this controller.
	pub primary_channel: IdeChannel,
	/// The second channel on this controller.
	pub secondary_channel: IdeChannel,
}
impl IdeController {
	/// Checks if a PCI device is an IDE controller, and if it is, returns the device.
	pub fn from_pci(device: &mut PciDevice) -> Option<Self> {
		// IDE controllers have a class of `MassStorageController` and subclass of `IDE`.
		if device.class()
			!= Some(Class::MassStorageController(
				MassStorageControllerSubclass::Ide,
			)) {
			return None;
		}

		// The first four bits of the programming interface byte determine the mode of the two
		// channels. The first bit sets if the primary controller is in compatibility or native mode -
		// 0 means compat, 1 means native. The second bit sets if the primary controller can be switched
		// between compatibility and native mode (0 = cannot be switched, 1 = can be switched) by writing
		// to the first bit. The third and fourth bits are identical to the first and second, except they
		// apply to the second channel instead of the first.
		//
		// A primary channel in compatibility mode uses CPU I/O ports `0x1F0-0x1F7` and `0x3F6` to communicate.
		// A secondary channel in compatibility mode uses CPU I/O ports `0x170-0x177` and `0x376` to communicate.
		// Channels in native mode have their I/O ports specified in their BAR.
		let prog_if = device.programming_interface()?;
		let primary_channel = if (prog_if & 0b0001) == 0 {
			IdeChannel::new(0x01F0, 0x03F6)
		} else {
			todo!("Non-compatibility IDE channels")
		};
		let secondary_channel = if (prog_if & 0b0100) == 0 {
			IdeChannel::new(0x0170, 0x0376)
		} else {
			todo!("Non-compatibility IDE channels")
		};

		Some(Self {
			primary_channel,
			secondary_channel,
		})
	}
}

/// Represents one of two channels on an IDE controller. Each channel can have up to two drives.
pub struct IdeChannel {
	/// The first CPU I/O port this channel uses.
	primary_io_port: u16,
	/// The second CPU I/O port this channel uses.
	secondary_io_port: u16,
	/// The currently selected disk on this channel. Each channel can have up to two drives,
	/// but only one can be used at a time.
	active_disk: IdeDisk,
}
impl IdeChannel {
	pub fn new(primary_io_port: u16, secondary_io_port: u16) -> Self {
		let mut this = Self {
			primary_io_port,
			secondary_io_port,
			active_disk: IdeDisk::Primary,
		};

		let drive: u8 = this.read_register(AtaRegister::DriveSelect);
		let active_disk = if drive & 0b0000_1000 == 0 {
			IdeDisk::Primary
		} else {
			IdeDisk::Secondary
		};

		this.active_disk = active_disk;
		this
	}

	/// Send an ATA command to the active drive on this channel. Note that although the LBA here
	/// is 64-bits, the actual LBA on the drive will either be 28 or 48 bits in length, depending
	/// on the command you send. This function does not verify the length of the LBA, you are
	/// responsible for that.
	pub fn send_command(&self, cmd: AtaCommand, lba: u64, sectors: u8) -> Result<(), AtaError> {
		let bytes = lba.to_le_bytes();
		self.write_register(AtaRegister::Lba0, bytes[0])?;
		self.write_register(AtaRegister::Lba1, bytes[1])?;
		self.write_register(AtaRegister::Lba2, bytes[2])?;
		self.write_register(AtaRegister::SectorCount, sectors)?;

		self.write_register(AtaRegister::Command, cmd as u8)
	}

	/// Enable or disable interrupt requests from the active drive on this channel.
	pub fn set_interrupts(&self, enabled: bool) {
		let mut val: u8 = self.read_register(AtaRegister::AltControl);
		// Port 2, register 0, bit 2
		// Set: Interrupts enabled
		// Unset: Interrupts disabled
		match enabled {
			true => val |= 0b0000_0010,
			false => val &= 0b1111_1101,
		}
		self.write_register(AtaRegister::AltControl, val).unwrap();
	}

	/// Switch which disk is active on this channel. This function does nothing if `disk`
	/// is already the active disk.
	pub fn set_disk(&mut self, disk: IdeDisk) {
		if disk != self.active_disk {
			let mut val: u8 = self.read_register(AtaRegister::DriveSelect);
			// Register 6, bit 4
			// Set: Use secondary drive
			// Unset: Use primary drive
			match disk {
				IdeDisk::Primary => val &= 0b1111_0111,
				IdeDisk::Secondary => val |= 0b0000_1000,
			}
			self.write_register(AtaRegister::DriveSelect, val).unwrap();
			self.active_disk = disk;
		}
	}
	/// See which disk is active on this channel.
	pub fn active_disk(&self) -> IdeDisk {
		self.active_disk
	}

	/// Read from one of the active disk's registers. This function works with
	/// both 8-bit and 16-bit registers via generics, but it doesn't check that
	/// you use the right size for a particular register - you are responsible
	/// for that.
	pub fn read_register<S: PortSize>(&self, register: AtaRegister) -> S {
		// Alternate registers are on the secondary I/O port
		let base_port = if register.is_alt() {
			self.secondary_io_port
		} else {
			self.primary_io_port
		};
		let register: u16 = register.into();

		S::read(base_port + register)
	}
	/// Write to one of the active disk's registers. This function works with
	/// both 8-bit and 16-bit registers via generics, but it doesn't check that
	/// you use the right size for a particular register - you are responsible
	/// for that.
	///
	/// This function will automatically check for and return ATA errors if it
	/// detects one. It also automatically blocks until the drive's `Busy` bit is clear.
	pub fn write_register<S: PortSize>(
		&self,
		register: AtaRegister,
		data: S,
	) -> Result<(), AtaError> {
		// Alternate registers are on the secondary I/O port
		let base_port = if register.is_alt() {
			self.secondary_io_port
		} else {
			self.primary_io_port
		};
		let register: u16 = register.into();

		S::write(base_port + register, data);

		// https://wiki.osdev.org/ATA_PIO_Mode#400ns_delays
		for _ in 0..15 {
			let _: u8 = self.read_register(AtaRegister::Status);
		}
		loop {
			let status: u8 = self.read_register(AtaRegister::Status);

			if status & AtaStatus::Error as u8 != 0 {
				let err_reg: u8 = self.read_register(AtaRegister::Error);
				for err in AtaError::VARIANTS {
					if err_reg & err as u8 != 0 {
						return Err(err);
					}
				}
				return Err(AtaError::Unknown);
			}

			if (status & AtaStatus::Busy as u8) == 0 {
				break;
			}
		}
		Ok(())
	}
}

/// This trait allows functions that work with CPU ports to work
/// with ports of different sizes. The idea is that a function
/// can take or return a [`PortSize`] as a generic, and use that
/// generic to read from/write to a CPU port. The generic will
/// then handle the port's size (8 bits, 16 bits, etc) automatically.
pub trait PortSize {
	/// Read from a CPU port.
	fn read(port: u16) -> Self;
	/// Write to a CPU port.
	fn write(port: u16, data: Self);
}
impl PortSize for u8 {
	fn read(port: u16) -> Self {
		let val;
		unsafe { asm!("in al, dx", in("dx") port, out("al") val) }
		val
	}
	fn write(port: u16, data: Self) {
		unsafe { asm!("out dx, al", in("dx") port, in("al") data) }
	}
}
impl PortSize for u16 {
	fn read(port: u16) -> Self {
		let val;
		unsafe { asm!("in ax, dx", in("dx") port, out("ax") val) }
		val
	}
	fn write(port: u16, data: Self) {
		unsafe { asm!("out dx, ax", in("dx") port, in("ax") data) }
	}
}
