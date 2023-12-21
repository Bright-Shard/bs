use core::fmt::{Arguments as FmtArgs, Write};

#[repr(packed)]
pub struct VgaTextChar {
    pub colour: u8,
    pub letter: u8,
}

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

pub static mut BUFFER: VgaTextBuffer = VgaTextBuffer { idx: 0 };

#[macro_export]
macro_rules! print {
    () => {};
    ($($arg:tt)*) => {
        unsafe { $crate::vga_text_buffer::BUFFER.write(format_args!($($arg)*)) }
    };
}
#[macro_export]
macro_rules! println {
    () => {
        unsafe { $crate::vga_text_buffer::BUFFER.write_byte('\n') }
    };
    ($($arg:tt)*) => {
        unsafe { $crate::vga_text_buffer::BUFFER.write(format_args!("{}\n", format_args!($($arg)*))) }
    };
}
