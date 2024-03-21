use exrs::variants;

/// ATA devices have a series of registers that are read or written to to interact
/// with the device. Each register is just an offset from the base CPU I/O port used
/// to interact with the device.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum AtaRegister {
	/// The data register is 16 bits long and stores data returned from the device.
	/// For example, data returned from a PIO read is stored in this register. You
	/// probably need to read this multiple times to get all the data - for example,
	/// a PIO read returns 512 bytes, so you have to read this 256 times to get all
	/// the data.
	Data,
	/// Stores a sequence of bit flags that indicate any errors that occurred. See the
	/// [`AtaError`] enum for those flags.
	Error,
	/// Unsure what this is for. The OSDev wiki says: "Used to control command
	/// specific interface features."
	Features,
	/// Stores the number of sectors to read from the disk.
	SectorCount,
	/// The first of three 16-bit registers storing a 48-bit LBA. The LBA selects which disk
	/// sector to read from, where a sector is a set of 512 bytes from the disk. An LBA of 0
	/// would select the first sector/first 512 bytes, an LBA of 1 would select the
	/// next sector/512 bytes, etc.
	///
	/// Technically, each disk can have its own sector size, but the most commonly used
	/// sector size is 512 bytes.
	///
	/// Also note that some older ATA commands only support a 28-bit LBA. The remaining 20
	/// LBA bits must be set to 0 if this is the case.
	Lba0,
	/// See [`Self::Lba0`].
	Lba1,
	/// See [`Self::Lba0`].
	Lba2,
	/// Used to select drive 1 or 2 on this ATA channel, and enable either CHS or LBA
	/// addressing.
	DriveSelect,
	/// Used to send a command to the ATA disk.
	Command,
	/// Has bitflags indicating if the disk has encountered an error, or finished
	/// processing a command, etc. See [`AtaStatus`] for this bitflags.
	Status,

	// Alt registers - these are read from the device's second base I/O port
	/// Used to enable/disable interrupt requests from the drive.
	AltControl,
	/// A duplicate of the other status register - the OSDev wiki says it
	/// doesn't affect interrupts.
	AltStatus,
	/// According to the OSDev wiki, "Provides drive select and head select information."
	DeviceAddress,
}
impl AtaRegister {
	/// There are two type of ATA registers - regular ones and alternate ones. Alternate
	/// registers are offsets from the drive's second CPU I/O port, while regular ones
	/// are offsets from the drive's first CPU I/O port.
	pub const fn is_alt(&self) -> bool {
		matches!(
			self,
			Self::AltControl | Self::AltStatus | Self::DeviceAddress
		)
	}
}
impl From<AtaRegister> for u16 {
	fn from(value: AtaRegister) -> Self {
		match value {
			AtaRegister::Data => 0x00,
			AtaRegister::Error | AtaRegister::Features => 0x01,
			AtaRegister::SectorCount => 0x02,
			AtaRegister::Lba0 => 0x03,
			AtaRegister::Lba1 => 0x04,
			AtaRegister::Lba2 => 0x05,
			AtaRegister::DriveSelect => 0x06,
			AtaRegister::Command | AtaRegister::Status => 0x07,

			AtaRegister::AltControl | AtaRegister::AltStatus => 0x02,
			AtaRegister::DeviceAddress => 0x03,
		}
	}
}

/// The bitflags in the status register ([`AtaRegister::Status`]).
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum AtaStatus {
	/// When set, indicates an error ocurred. The exact error can be read from the error register
	/// ([`AtaRegister::Error`]).
	Error = 1 << 0,
	/// According the ATA-8 spec, this bit's meaning depends on context. According to the OSDev wiki,
	/// when this bit is set, it indicates that the drive is ready to transfer PIO data for a read/write.
	DataRequest = 1 << 4,
	/// According to the ATA-8 spec, indicates some critical error that could affect data integrity.
	/// When this bit gets set, the drive does not accept any more commands. This bit can only be
	/// cleared by power cycling the drive.
	DeviceFault = 1 << 5,
	/// According to the OSDev wiki: "Bit is clear when drive is spun down, or after an error. Set otherwise."
	DeviceReady = 1 << 6,
	/// What this bit means depends on context. The most common use is waiting for
	/// it to clear after sending a command, which indicates that the command finished
	/// running.
	Busy = 1 << 7,
}

/// The bitflags in the error register ([`AtaRegister::Error`]). These are taken from the OSDev wiki.
#[variants]
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum AtaError {
	NoAddressMark = 0x01,
	Track0NotFound = 0x02,
	CommandAborted = 0b0000_00100,
	MediaChangeRequest = 0x08,
	IdMarkNotFound = 0x10,
	MediaChanged = 0x20,
	UncorrectableData = 0x40,
	BadBlock = 0x80,
	Unknown,
}

/// The commands that can be sent to an ATA device.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum AtaCommand {
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

/// Represents a disk in an IDE channel. Each channel can have two drives.
///
/// These are usually called "master" and "slave", but those terms imply that
/// one drive can control the other, which isn't true, and are also ethically questionable.
/// So while some online docs may call these master or slave, I will use the terms
/// primary and secondary.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum IdeDisk {
	Primary,
	Secondary,
}
