//! Types for interrupt handling. Interrupts are given to the CPU when
//! certain events happen, like a key being pressed or a click ticking.
//! This is currently incomplete.
//!
//! Resources:
//! - https://wiki.osdev.org/Interrupt_Descriptor_Table
//! - https://wiki.osdev.org/Interrupt_Service_Routines

/// The Interrupt Descriptor Table. Stores a list of interrupt descriptors,
/// which all store handlers for interrupts.
#[repr(transparent)]
pub struct Idt<const LEN: usize> {
	/// All the interrupts in this IDT.
	pub interrupts: [InterruptDescriptor; LEN],
}

/// Describes a handler for a specific CPU interrupt.
#[repr(packed)]
#[derive(Clone)]
pub struct InterruptDescriptor {
	/// An offset to an Interrupt Service Routine, which is the function
	/// that gets called to handle this interrupt.
	pub offset1: u16,
	/// A segment selector for this handler: https://wiki.osdev.org/Segment_Selector
	pub segment: u16,
	// Actually a u3 followed by 5 reserved bits
	pub stack_table: u8,
	/// Defines a type for this interrupt handler, its privilege level, and if it's
	/// enabled.
	pub attributes: u8,
	/// More bytes to `offset1`.
	pub offset2: u16,
	/// More bytes to `offset1`.
	pub offset3: u32,
	_reserved: u32,
}
impl InterruptDescriptor {
	pub const NULL: Self = Self {
		offset1: 0,
		segment: 0,
		stack_table: 0,
		attributes: 0,
		offset2: 0,
		offset3: 0,
		_reserved: 0,
	};
}

/// Stores a pointer to the IDT. This is stored by the CPU instead
/// of the actual IDT.
#[repr(packed)]
pub struct IdtDescriptor {
	pub size: u16,
	pub offset: u64,
}
