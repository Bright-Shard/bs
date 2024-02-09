//! Adds the `print!` and `println!` macros, just like you'd find in standard Rust.
//! If using the BIOS feature, this uses int 0x10 to print characters.
//! Otherwise, this uses VGA text mode.

use core::fmt::{Arguments as FmtArgs, Write};

pub static mut GLOBAL_PRINTER: VgaTextBuffer = VgaTextBuffer { idx: 0 };

#[derive(Default)]
pub struct VgaTextBuffer {
	pub idx: usize,
}
impl VgaTextBuffer {
	const BUFFER: *mut [VgaTextChar; 8_000] = 0xB8000 as *mut _;
	const NUM_ROWS: usize = 25;
	const NUM_COLUMNS: usize = 80;
	const LEN: usize = Self::NUM_ROWS * Self::NUM_COLUMNS;

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
	}

	/// Clears the whole VGA buffer, making the screen black.
	pub fn clear(&mut self) {
		let buffer = unsafe { &mut *Self::BUFFER };

		// For some reason, this while loop gets compiled down to *much* less
		// code than a for loop does. We need a small binary size for the
		// bootstrapper, so we use a while loop instead of a for loop.
		let mut idx = 0;
		while idx != Self::LEN {
			buffer[idx].letter = 0;
			buffer[idx].colour = 0;
			idx += 1;
		}

		self.idx = 0;
	}

	#[inline(always)]
	pub fn write(&mut self, args: FmtArgs) {
		self.write_fmt(args).unwrap()
	}
	#[inline(always)]
	pub fn write_nofmt(&mut self, s: &str) {
		s.as_bytes().iter().for_each(|byte| self.write_byte(*byte));
	}
}
impl Write for VgaTextBuffer {
	fn write_str(&mut self, s: &str) -> core::fmt::Result {
		self.write_nofmt(s);
		Ok(())
	}
}

#[repr(packed)]
pub struct VgaTextChar {
	pub letter: u8,
	pub colour: u8,
}

#[macro_export]
macro_rules! print {
    () => {};
    ($($arg:tt)*) => {
        unsafe { $crate::printing::GLOBAL_PRINTER.write(format_args!($($arg)*)) }
    };
}
#[macro_export]
macro_rules! println {
    () => {
        unsafe { $crate::printing::GLOBAL_PRINTER.write_byte(b'\n') }
    };
    ($($arg:tt)*) => {
        unsafe { $crate::printing::GLOBAL_PRINTER.write(format_args!("{}\n", format_args!($($arg)*))) }
    };
}

#[macro_export]
macro_rules! print_nofmt {
	() => {};
	($str:literal) => {
		unsafe { $crate::printing::GLOBAL_PRINTER.write_nofmt($str) }
	};
}
#[macro_export]
macro_rules! println_nofmt {
	() => {
		unsafe { $crate::printing::GLOBAL_PRINTER.write_byte(b'\n') }
	};
	($str:literal) => {
		unsafe { $crate::printing::GLOBAL_PRINTER.write_nofmt($str) }
	};
}
