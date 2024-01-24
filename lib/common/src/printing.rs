//! Adds the `print!` and `println!` macros, just like you'd find in standard Rust.
//! If using the BIOS feature, this uses int 0x10 to print characters.
//! Otherwise, this uses VGA text mode.

#[cfg(not(feature = "bios"))]
mod vga {
	use core::fmt::{Arguments as FmtArgs, Write};

	pub static mut BUFFER: VgaTextBuffer = VgaTextBuffer { idx: 0 };

	pub struct VgaTextBuffer {
		pub idx: usize,
	}
	impl VgaTextBuffer {
		pub fn write_byte(&mut self, byte: u8) {
			let buffer: &mut [VgaTextChar; 8_000] = unsafe { &mut *(0x8b000 as *mut _) };
			buffer[self.idx].letter = byte;
			self.idx += 1;
		}

		#[inline]
		pub fn write(&mut self, args: FmtArgs) {
			self.write_fmt(args).unwrap()
		}
	}
	impl Write for VgaTextBuffer {
		fn write_str(&mut self, s: &str) -> core::fmt::Result {
			s.bytes().for_each(|byte| self.write_byte(byte));
			Ok(())
		}
	}

	#[repr(packed)]
	pub struct VgaTextChar {
		pub colour: u8,
		pub letter: u8,
	}
}

#[cfg(feature = "bios")]
mod bios {
	use core::{
		arch::asm,
		fmt::{Arguments as FmtArgs, Write},
	};

	pub static mut BUFFER: BiosPrinter = BiosPrinter;

	pub struct BiosPrinter;
	impl BiosPrinter {
		pub fn write_byte(byte: u8) {
			unsafe {
				asm!(
					"mov ah, 0x0e",
					"int 0x10",
					in("al") byte,
				);
			}

			// Newlines don't automatically go back to the first column, so
			// here we add a carriage return as well to do that.
			if byte == b'\n' {
				Self::write_byte(b'\r');
			}
		}

		#[inline]
		pub fn write(&mut self, args: FmtArgs) {
			self.write_fmt(args).unwrap();
		}
	}
	impl Write for BiosPrinter {
		fn write_str(&mut self, s: &str) -> core::fmt::Result {
			s.bytes().for_each(Self::write_byte);
			Ok(())
		}
	}
}

#[cfg(feature = "bios")]
pub use bios::*;
#[cfg(not(feature = "bios"))]
pub use vga::*;

#[macro_export]
macro_rules! print {
    () => {};
    ($($arg:tt)*) => {
        unsafe { $crate::printing::BUFFER.write(format_args!($($arg)*)) }
    };
}
#[macro_export]
macro_rules! println {
    () => {
        unsafe { $crate::printing::BUFFER.write_byte('\n') }
    };
    ($($arg:tt)*) => {
        unsafe { $crate::printing::BUFFER.write(format_args!("{}\n", format_args!($($arg)*))) }
    };
}
