use {
    core::fmt::{self, Write},
    spin::Mutex,
};

pub struct VgaTextBuffer {
    pub idx: usize,
    pub style: u8,
}
impl VgaTextBuffer {
    pub const BUFFER: *mut [u8; 16_000] = (0xb8000 as *mut u8).cast();
    pub const ROW_MAX: usize = 25;
    pub const COL_MAX: usize = 80 * 2;
    pub const MAX: usize = Self::ROW_MAX * Self::COL_MAX;

    pub fn move_to(&mut self, idx: usize) {
        if idx < Self::MAX {
            self.idx = idx;
        }
    }

    pub fn write_byte(&mut self, byte: u8) {
        if byte == b'\n' {
            self.idx += Self::COL_MAX - (self.idx % Self::COL_MAX);
        } else {
            unsafe {
                (*Self::BUFFER)[self.idx] = byte;
                (*Self::BUFFER)[self.idx + 1] = self.style;
                self.idx += 2;
            }
        }

        if self.idx >= Self::MAX {
            let buffer = unsafe { &mut *Self::BUFFER };
            buffer[0] = b'z';
            for byte in buffer.iter_mut() {
                *byte = 0;
            }
            self.idx = 0;
            buffer[0] = b'Z';
        }
    }
}

impl Write for VgaTextBuffer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        s.bytes().for_each(|byte| self.write_byte(byte));
        Ok(())
    }
}

pub static SCREEN: Mutex<VgaTextBuffer> = Mutex::new(VgaTextBuffer {
    idx: 0,
    style: 0b0000_1111,
});

pub fn _screen_fmt_print(args: fmt::Arguments) {
    x86_64::instructions::interrupts::without_interrupts(|| SCREEN.lock().write_fmt(args)).unwrap();
}

#[macro_export]
macro_rules! print {
    () => {};
    ($($arg:tt)*) => {
        $crate::vga::_screen_fmt_print(format_args!($($arg)*))
    };
}
pub use print;

#[macro_export]
macro_rules! println {
    () => {
        $crate::vga::print!("\n")
    };

    ($($arg:tt)*) => {
        $crate::vga::print!("{}\n", format_args!($($arg)*))
    };
}
pub use println;
