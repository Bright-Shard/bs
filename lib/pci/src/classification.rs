//! Enums for PCI device classifications, according to: https://wiki.osdev.org/PCI#Class_Codes

#[derive(Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Class {
	Unclassified(UnclassifiedSubclass) = 0,
	MassStorageController(MassStorageControllerSubclass) = 1,
	NetworkController(NetworkControllerSubclass) = 2,
	DisplayController(DisplayControllerSubclass) = 3,
	MultimediaController(MultimediaControllerSubclass) = 4,
	MemoryController(MemoryControllerSubclass) = 5,
	Bridge(BridgeSubclass) = 6,
	SimpleCommunicationController(SimpleCommunicationControllerSubclass) = 7,
	BaseSystemPeripheral(BaseSystemPeripheralSubclass) = 8,
	InputDeviceController(InputDeviceControllerSubclass) = 9,
	DockingStation(DockingStationSubclass) = 10,
	Processor(ProcessorSubclass) = 11,
	SerialBusController(SerialBusControllerSubclass) = 12,
	WirelessController(WirelessControllerSubclass) = 13,
	IntelligentController(IntelligentControllerSubclass) = 14,
	SatelliteCommunicationController(SatelliteCommunicationControllerSubclass) = 15,
	EncryptionController(EncryptionControllerSubclass) = 16,
	SignalProcessingController(SignalProcessingControllerSubclass) = 17,
	ProcessingController = 18,
	NonEssentialInstrumentation = 19,
	CoProcessor = 0x40,
	Unassigned = 0xFF,
}

#[derive(Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum UnclassifiedSubclass {
	NonVgaCompatible = 0,
	VgaCompatible = 1,
}
#[derive(Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum MassStorageControllerSubclass {
	ScsiBus = 0,
	Ide = 1,
	FloppyDisk = 2,
	IpiBus = 3,
	Raid = 4,
	Ata = 5,
	SerialAta = 6,
	SerialAttachedScsi = 7,
	NonVolatileMemory = 8,
	Other = 0x80,
}
#[derive(Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum NetworkControllerSubclass {
	Ethernet = 0,
	TokenRing = 1,
	Fddi = 2,
	Atm = 3,
	Isdn = 4,
	WorldFip = 5,
	PicMg = 6,
	Infiniband = 7,
	Fabric = 8,
	Other = 0x80,
}
#[derive(Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum DisplayControllerSubclass {
	VgaCompatible = 0,
	Xga = 1,
	NonVga3d = 2,
	Other = 0x80,
}
#[derive(Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum MultimediaControllerSubclass {
	MultimediaVideo = 0,
	MultimediaAudio = 1,
	ComputerTelephony = 2,
	Audio = 3,
	Other = 0x80,
}
#[derive(Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum MemoryControllerSubclass {
	Ram = 0,
	Flash = 1,
	Other = 0x80,
}
#[derive(Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum BridgeSubclass {
	Host = 0,
	Isa = 1,
	Eisa = 2,
	Mca = 3,
	PciToPci = 4,
	Pcmcia = 5,
	NuBus = 6,
	CardBus = 7,
	RaceWay = 8,
	PciToPciSemiTransparent = 9,
	InfinibandToPci = 10,
	Other = 0x80,
}
#[derive(Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum SimpleCommunicationControllerSubclass {
	Serial = 0,
	Parallel = 1,
	MultiportSerial = 2,
	Modem = 3,
	Ieee488 = 4,
	SmartCard = 5,
	Other = 0x80,
}
#[derive(Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum BaseSystemPeripheralSubclass {
	Pic = 0,
	DmaController = 1,
	Timer = 2,
	RtcController = 3,
	PciHotPlugController = 4,
	SdHostController = 5,
	Iommu = 6,
	Other = 0x80,
}
#[derive(Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum InputDeviceControllerSubclass {
	Keyboard = 0,
	DigitizerPen = 1,
	Mouse = 2,
	Scanner = 3,
	Gameport = 4,
	Other = 0x80,
}
#[derive(Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum DockingStationSubclass {
	Generic = 1,
	Other = 0x80,
}
#[derive(Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ProcessorSubclass {
	Processor386 = 0,
	Processor486 = 1,
	Pentium = 2,
	PentiumPro = 3,
	Alpha = 16,
	PowerPc = 32,
	Mips = 48,
	CoProcessor = 64,
	Other = 0x80,
}
#[derive(Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum SerialBusControllerSubclass {
	FireWire = 0,
	AccessBus = 1,
	Ssa = 2,
	UsbController = 3,
	Fibre = 4,
	SmBus = 5,
	Infiniband = 6,
	Ipmi = 7,
	Sercos = 8,
	CanBus = 9,
	Other = 0x80,
}
#[derive(Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum WirelessControllerSubclass {
	IRdaCompatible = 0,
	ConsumerIr = 1,
	Rf = 16,
	Bluetooth = 17,
	Broadband = 18,
	Ethernet8021a = 32,
	Ethernet8021b = 33,
	Other = 0x80,
}
#[derive(Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum IntelligentControllerSubclass {
	I20 = 0x80,
}
#[derive(Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum SatelliteCommunicationControllerSubclass {
	Tv = 1,
	Audio = 2,
	Voice = 3,
	Data = 4,
}
#[derive(Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum EncryptionControllerSubclass {
	NetworkAndComputing = 0,
	Entertainment = 16,
	Other = 0x80,
}
#[derive(Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum SignalProcessingControllerSubclass {
	DpioModules = 0,
	PerformaceCounters = 1,
	CommunicationSynchronizer = 16,
	SignalProcessingManagement = 32,
	Other = 0x80,
}
impl Class {
	pub fn from_bytes(class: u8, subclass: u8) -> Option<Self> {
		Some(match class {
			0 => Class::Unclassified(match subclass {
				0 => UnclassifiedSubclass::NonVgaCompatible,
				1 => UnclassifiedSubclass::VgaCompatible,
				_ => return None,
			}),
			1 => Class::MassStorageController(match subclass {
				0 => MassStorageControllerSubclass::ScsiBus,
				1 => MassStorageControllerSubclass::Ide,
				2 => MassStorageControllerSubclass::FloppyDisk,
				3 => MassStorageControllerSubclass::IpiBus,
				4 => MassStorageControllerSubclass::Raid,
				5 => MassStorageControllerSubclass::Ata,
				6 => MassStorageControllerSubclass::SerialAta,
				7 => MassStorageControllerSubclass::SerialAttachedScsi,
				8 => MassStorageControllerSubclass::NonVolatileMemory,
				0x80 => MassStorageControllerSubclass::Other,
				_ => return None,
			}),
			2 => Class::NetworkController(match subclass {
				0 => NetworkControllerSubclass::Ethernet,
				1 => NetworkControllerSubclass::TokenRing,
				2 => NetworkControllerSubclass::Fddi,
				3 => NetworkControllerSubclass::Atm,
				4 => NetworkControllerSubclass::Isdn,
				5 => NetworkControllerSubclass::WorldFip,
				6 => NetworkControllerSubclass::PicMg,
				7 => NetworkControllerSubclass::Infiniband,
				8 => NetworkControllerSubclass::Fabric,
				0x80 => NetworkControllerSubclass::Other,
				_ => return None,
			}),
			3 => Class::DisplayController(match subclass {
				0 => DisplayControllerSubclass::VgaCompatible,
				1 => DisplayControllerSubclass::Xga,
				2 => DisplayControllerSubclass::NonVga3d,
				0x80 => DisplayControllerSubclass::Other,
				_ => return None,
			}),
			4 => Class::MultimediaController(match subclass {
				0 => MultimediaControllerSubclass::MultimediaVideo,
				1 => MultimediaControllerSubclass::MultimediaAudio,
				2 => MultimediaControllerSubclass::ComputerTelephony,
				3 => MultimediaControllerSubclass::Audio,
				0x80 => MultimediaControllerSubclass::Other,
				_ => return None,
			}),
			5 => Class::MemoryController(match subclass {
				0 => MemoryControllerSubclass::Ram,
				1 => MemoryControllerSubclass::Flash,
				0x80 => MemoryControllerSubclass::Other,
				_ => return None,
			}),
			6 => Class::Bridge(match subclass {
				0 => BridgeSubclass::Host,
				1 => BridgeSubclass::Isa,
				2 => BridgeSubclass::Eisa,
				3 => BridgeSubclass::Mca,
				4 => BridgeSubclass::PciToPci,
				5 => BridgeSubclass::Pcmcia,
				6 => BridgeSubclass::NuBus,
				7 => BridgeSubclass::CardBus,
				8 => BridgeSubclass::RaceWay,
				9 => BridgeSubclass::PciToPciSemiTransparent,
				10 => BridgeSubclass::InfinibandToPci,
				0x80 => BridgeSubclass::Other,
				_ => return None,
			}),
			7 => Class::SimpleCommunicationController(match subclass {
				0 => SimpleCommunicationControllerSubclass::Serial,
				1 => SimpleCommunicationControllerSubclass::Parallel,
				2 => SimpleCommunicationControllerSubclass::MultiportSerial,
				3 => SimpleCommunicationControllerSubclass::Modem,
				4 => SimpleCommunicationControllerSubclass::Ieee488,
				5 => SimpleCommunicationControllerSubclass::SmartCard,
				0x80 => SimpleCommunicationControllerSubclass::Other,
				_ => return None,
			}),
			8 => Class::BaseSystemPeripheral(match subclass {
				0 => BaseSystemPeripheralSubclass::Pic,
				1 => BaseSystemPeripheralSubclass::DmaController,
				2 => BaseSystemPeripheralSubclass::Timer,
				3 => BaseSystemPeripheralSubclass::RtcController,
				4 => BaseSystemPeripheralSubclass::PciHotPlugController,
				5 => BaseSystemPeripheralSubclass::SdHostController,
				6 => BaseSystemPeripheralSubclass::Iommu,
				0x80 => BaseSystemPeripheralSubclass::Other,
				_ => return None,
			}),
			9 => Class::InputDeviceController(match subclass {
				0 => InputDeviceControllerSubclass::Keyboard,
				1 => InputDeviceControllerSubclass::DigitizerPen,
				2 => InputDeviceControllerSubclass::Mouse,
				3 => InputDeviceControllerSubclass::Scanner,
				4 => InputDeviceControllerSubclass::Gameport,
				0x80 => InputDeviceControllerSubclass::Other,
				_ => return None,
			}),
			10 => Class::DockingStation(match subclass {
				1 => DockingStationSubclass::Generic,
				0x80 => DockingStationSubclass::Other,
				_ => return None,
			}),
			11 => Class::Processor(match subclass {
				0 => ProcessorSubclass::Processor386,
				1 => ProcessorSubclass::Processor486,
				2 => ProcessorSubclass::Pentium,
				3 => ProcessorSubclass::PentiumPro,
				16 => ProcessorSubclass::Alpha,
				32 => ProcessorSubclass::PowerPc,
				48 => ProcessorSubclass::Mips,
				64 => ProcessorSubclass::CoProcessor,
				0x80 => ProcessorSubclass::Other,
				_ => return None,
			}),
			12 => Class::SerialBusController(match subclass {
				0 => SerialBusControllerSubclass::FireWire,
				1 => SerialBusControllerSubclass::AccessBus,
				2 => SerialBusControllerSubclass::Ssa,
				3 => SerialBusControllerSubclass::UsbController,
				4 => SerialBusControllerSubclass::Fibre,
				5 => SerialBusControllerSubclass::SmBus,
				6 => SerialBusControllerSubclass::Infiniband,
				7 => SerialBusControllerSubclass::Ipmi,
				8 => SerialBusControllerSubclass::Sercos,
				9 => SerialBusControllerSubclass::CanBus,
				0x80 => SerialBusControllerSubclass::Other,
				_ => return None,
			}),
			13 => Class::WirelessController(match subclass {
				0 => WirelessControllerSubclass::IRdaCompatible,
				1 => WirelessControllerSubclass::ConsumerIr,
				16 => WirelessControllerSubclass::Rf,
				17 => WirelessControllerSubclass::Bluetooth,
				18 => WirelessControllerSubclass::Broadband,
				32 => WirelessControllerSubclass::Ethernet8021a,
				33 => WirelessControllerSubclass::Ethernet8021b,
				0x80 => WirelessControllerSubclass::Other,
				_ => return None,
			}),
			14 => Class::IntelligentController(match subclass {
				0x80 => IntelligentControllerSubclass::I20,
				_ => return None,
			}),
			15 => Class::SatelliteCommunicationController(match subclass {
				1 => SatelliteCommunicationControllerSubclass::Tv,
				2 => SatelliteCommunicationControllerSubclass::Audio,
				3 => SatelliteCommunicationControllerSubclass::Voice,
				4 => SatelliteCommunicationControllerSubclass::Data,
				_ => return None,
			}),
			16 => Class::SignalProcessingController(match subclass {
				0 => SignalProcessingControllerSubclass::DpioModules,
				1 => SignalProcessingControllerSubclass::PerformaceCounters,
				16 => SignalProcessingControllerSubclass::CommunicationSynchronizer,
				32 => SignalProcessingControllerSubclass::SignalProcessingManagement,
				0x80 => SignalProcessingControllerSubclass::Other,
				_ => return None,
			}),
			17 => Class::ProcessingController,
			18 => Class::NonEssentialInstrumentation,
			0x40 => Class::CoProcessor,
			0xFF => Class::Unassigned,
			_ => return None,
		})
	}
}

/// The PCI device's vendor. Vendor IDs are allocated by PCI-Sig here: https://pcisig.com/membership/member-companies
/// TODO: Port vendors over (oh my god are there a lot...)
#[repr(u16)]
#[derive(Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Vendor {
	AdvancedMicroDevices = 0x1022,
}
impl TryFrom<u16> for Vendor {
	type Error = ();

	fn try_from(value: u16) -> Result<Self, Self::Error> {
		Ok(match value {
			0x1022 => Self::AdvancedMicroDevices,
			_ => return Err(()),
		})
	}
}

/// Metadata in a PCI configuration space header.
pub struct HeaderMeta {
	/// If this device has multiple functions.
	pub multi_function: bool,
	/// The configuration space should be interpreted differently depending on the type of its
	/// header.
	pub kind: HeaderType,
}
impl TryFrom<u8> for HeaderMeta {
	type Error = ();

	fn try_from(value: u8) -> Result<Self, ()> {
		// If bit 7 is set, this device has multiple functions
		let multi_function = (value & (1 << 7)) != 0;

		// First two bits indicate device type
		let kind = match value & 0b0000_0011 {
			0 => HeaderType::General,
			1 => HeaderType::PciToPci,
			2 => HeaderType::PciToCardbus,
			_ => unreachable!(),
		};

		Ok(Self {
			multi_function,
			kind,
		})
	}
}
#[derive(Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum HeaderType {
	/// A PCI header for a generic PCI device.
	General = 0,
	/// A PCI header for a PCI to PCI bridge.
	PciToPci = 1,
	/// A PCI header for a PCI to CardBus bridge.
	PciToCardbus = 2,
	Unknown,
}
