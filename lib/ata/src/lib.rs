#![no_std]

use {
	core::arch::asm,
	pci::{
		classification::{Class, MassStorageControllerSubclass},
		PciDevice,
	},
};

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
		// Channels in native mode have their I/O ports set in their BAR.
		let prog_if = device.programming_interface()?;
		let primary_channel = if (prog_if & 0b0001) == 0 {
			IdeChannel {
				primary_io_port: 0x01F0,
				secondary_io_port: 0x03F6,
			}
		} else {
			todo!("Non-compatibility IDE channels")
		};
		let secondary_channel = if (prog_if & 0b0100) == 0 {
			IdeChannel {
				primary_io_port: 0x0170,
				secondary_io_port: 0x0376,
			}
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
}
impl IdeChannel {
	pub fn send_command(&self, cmd: Command) {
		unsafe { asm!("out dx, al", in("dx") self.primary_io_port + 7, in("al") cmd as u8) }
	}

	pub fn set_interrupts(&self, enabled: bool) {
		let mut val = read_io_port(self.secondary_io_port);
		// Port 2, register 0, bit 2
		// Set: Interrupts enabled // Unset: Interrupts disabled
		match enabled {
			true => val |= 0b0000_0010,
			false => val &= 0b1111_1101,
		}
		write_io_port(self.secondary_io_port, val);
	}

	pub fn set_disk(&self, drive: IdeDisk) {
		let mut val = read_io_port(self.primary_io_port + 6);
		// Register 6, bit 4
		// Set: Use secondary drive // Unset: Use primary drive
		match drive {
			IdeDisk::Primary => val &= 0b1111_0111,
			IdeDisk::Secondary => val |= 0b0000_1000,
		}
		write_io_port(self.primary_io_port + 6, val)
	}
}

fn read_io_port(port: u16) -> u8 {
	let mut val;
	unsafe { asm!("in al, dx", in("dx") port, out("al") val) }
	val
}
fn write_io_port(port: u16, val: u8) {
	unsafe { asm!("out dx, al", in("dx") port, in("al") val) }
}

// /// Each IDE controller has two channels. Each channel can either be in compatibility
// /// or native mode, and has two devices.
// pub enum IdeChannel<'a> {
// 	/// An IDE controller in compatibility mode. It has hardcoded CPU I/O ports, which are
// 	/// `0x1F0-0x1F7`/`0x3F6` for the primary channel and `0x170-0x177`/`0x376` for the secondary
// 	/// channel. It also has hardcoded IRQs, which are 14 for the primary channel and 15 for the
// 	/// secondary channel.
// 	Compatibility(&'a mut PciDevice),
// 	Native(&'a mut PciDevice),
// }

pub enum AtaRegister {
	Data = 0x00,
	ErrorOrFeatures = 0x01,
	Seccount0 = 0x02,
	Lba0 = 0x03,
	Lba1 = 0x04,
	Lba2 = 0x05,
	DriveSelect = 0x06,
	CommandOrStatus = 0x07,
	Seccount1 = 0x08,
	Lba3 = 0x09,
	Lba4 = 0x0A,
	Lba5 = 0x0B,
}
pub enum AltAtaRegister {
	ControlOrStatus = 0x02,
	DevAddress = 0x03,
}

pub enum DeviceStatus {
	Error = 0x01,
	Index = 0x02,
	CorrectedData = 0x04,
	DataRequestReady = 0x08,
	SeekComplete = 0x10,
	WriteFault = 0x20,
	Ready = 0x40,
	Busy = 0x80,
}

pub enum Error {
	NoAddressMark = 0x01,
	Track0NotFound = 0x02,
	CommandAborted = 0x04,
	MediaChangeRequest = 0x08,
	IdMarkNotFound = 0x10,
	MediaChanged = 0x20,
	UncorrectableData = 0x40,
	BadBlock = 0x80,
}

pub enum Command {
	// bro wtf are these values ;-;
	ReadPio = 0x20,
	ReadPioExtended = 0x24,
	ReadDma = 0xC8,
	ReadDmaExtended = 0x25,
	WritePio = 0x30,
	WritePioExtended = 0x34,
	WriteDma = 0xCA,
	WriteDmaExtended = 0x35,
	CacheFlush = 0xE7,
	CacheFlushExtended = 0xEA,
	Packet = 0xA0,
	IdentifyPacket = 0xA1,
	Identify = 0xEC,
}

pub enum AtapiCommand {
	Read = 0xA8,
	Eject = 0x1B,
}

/// Represents a disk in an IDE channel.
///
/// These are usually called "master" and "slave", but even setting ethics aside,
/// these terms don't actually make sense because neither drive can control the
/// other - they're literally just two separate drives. So I refer to them as
/// primary and secondary drives instead, which should correct the misleading names.
pub enum IdeDisk {
	Primary,
	Secondary,
}
