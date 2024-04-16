//! Adds the `print!` and `println!` macros, just like you'd find in standard Rust.
//! If using the BIOS feature, this uses int 0x10 to print characters.
//! Otherwise, this uses VGA text mode.

use core::{fmt::Write, ptr::addr_of_mut};

pub static mut GLOBAL_PRINTER: Printer = Printer { idx: 0 };

#[derive(Default)]
pub struct Printer {
	pub idx: usize,
}
#[allow(dead_code)] // Some consts are only used with certain crate features
impl Printer {
	const BUFFER: *mut [VgaTextChar; 8_000] = 0xB8000 as *mut _;
	const NUM_ROWS: usize = 25;
	const NUM_COLUMNS: usize = 80;
	const LEN: usize = Self::NUM_ROWS * Self::NUM_COLUMNS;

	pub fn get_global<'a>() -> &'a mut Self {
		unsafe { &mut *addr_of_mut!(GLOBAL_PRINTER) }
	}

	/// Prints one byte to the screen.
	pub fn write_byte(&mut self, byte: u8) {
		match byte {
			b'\n' => self.idx += Self::NUM_COLUMNS - (self.idx % Self::NUM_COLUMNS),
			b'\r' => self.idx -= self.idx % Self::NUM_COLUMNS,
			byte => {
				let buffer = unsafe { &mut *Self::BUFFER };
				buffer[self.idx].letter = byte;
				buffer[self.idx].colour = 0b0000_1111;
				self.idx += 1;
			}
		}
		// If text is going off the screen, bump it back up
		if self.idx >= Self::NUM_ROWS * Self::NUM_COLUMNS {
			self.bump_screen();
		}
	}

	/// Clears the whole VGA buffer, making the screen black.
	pub fn clear(&mut self) {
		let buffer = unsafe { &mut *Self::BUFFER };

		for char in buffer {
			char.letter = 0;
			char.colour = 0;
		}

		self.idx = 0;
	}

	// TODO: Rename
	// This function moves the text up on the screen to make room for more
	pub fn bump_screen(&mut self) {
		// Move text up to row above
		let buffer = unsafe { &mut *Self::BUFFER };
		for row in 1..Self::NUM_ROWS {
			for col in 0..Self::NUM_COLUMNS {
				let current_idx = row * Self::NUM_COLUMNS + col;
				let target_idx = current_idx - Self::NUM_COLUMNS;
				buffer[target_idx] = buffer[current_idx];
			}
		}
		// Clear last row
		for col in 0..Self::NUM_COLUMNS {
			let idx = (Self::NUM_ROWS - 1) * Self::NUM_COLUMNS + col;
			buffer[idx].letter = 0;
			buffer[idx].colour = 0;
		}
		// Reset idx
		self.idx = (Self::NUM_ROWS - 1) * Self::NUM_COLUMNS;
	}
}
impl Write for Printer {
	fn write_str(&mut self, s: &str) -> core::fmt::Result {
		s.bytes().for_each(|byte| self.write_byte(byte));

		Ok(())
	}
}

#[repr(packed)]
#[derive(Clone, Copy)]
pub struct VgaTextChar {
	pub letter: u8,
	pub colour: u8,
}

#[macro_export]
macro_rules! print {
    () => {};
    ($($arg:tt)*) => {{
		use core::fmt::Write;

        $crate::printing::Printer::get_global().write_fmt(format_args!($($arg)*)).unwrap();
    }};
}
#[macro_export]
macro_rules! println {
    () => {
        $crate::printing::Printer::get_global().write_byte(b'\n')
    };
    ($($arg:tt)*) => {{
		use core::fmt::Write;

        $crate::printing::Printer::get_global().write_fmt(format_args!("{}\n", format_args!($($arg)*))).unwrap();
    }};
}
