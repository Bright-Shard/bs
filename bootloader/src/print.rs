//! Adds the `print!` and `println!` macros, just like you'd find in standard Rust.

use core::{
    arch::asm,
    fmt::{Arguments, Result, Write},
};

pub struct Printer;
impl Printer {
    pub fn print_byte(byte: u8) {
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
            Self::print_byte(b'\r');
        }
    }
}
impl Write for Printer {
    #[inline]
    fn write_str(&mut self, s: &str) -> Result {
        s.bytes().for_each(Self::print_byte);

        Ok(())
    }
}

#[inline]
pub fn print(args: Arguments) -> Result {
    Printer.write_fmt(args)
}

#[macro_export]
macro_rules! print {
    () => {};
    ($($arg:tt)*) => {
        $crate::print::print(format_args!($($arg)*)).unwrap()
    };
}

#[macro_export]
macro_rules! println {
    () => {
        print!("\n")
    };

    ($($arg:tt)*) => {
        print!("{}\n", format_args!($($arg)*))
    };
}
