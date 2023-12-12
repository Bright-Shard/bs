//! Adds the `print!` and `println!` macros, just like you'd find in standard Rust.

use core::{
    arch::asm,
    fmt::{Arguments, Result, Write},
};

pub struct Printer;
impl Write for Printer {
    #[inline]
    fn write_str(&mut self, s: &str) -> Result {
        s.bytes().for_each(|letter| unsafe {
            asm!(
                "mov ah, 0x0e",
                "int 0x10",
                in("al") letter,
            );
        });

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
        print!("\n\r")
    };

    ($($arg:tt)*) => {
        print!("{}\n\r", format_args!($($arg)*))
    };
}
