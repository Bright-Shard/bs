use {
    crate::{kbhandler, prelude::*},
    pc_keyboard::{layouts::Us104Key, DecodedKey, HandleControl, Keyboard, ScancodeSet1},
    pic8259::ChainedPics,
    spin::{Mutex, Once},
    x86_64::{
        instructions::port::Port,
        structures::idt::{InterruptDescriptorTable, InterruptStackFrame},
    },
};

#[repr(u8)]
pub enum Interrupts {
    Timer = 32,
    Keyboard = 33,
}

pub static IDT: Once<InterruptDescriptorTable> = Once::new();
pub static KEYBOARD: Mutex<Keyboard<pc_keyboard::layouts::Us104Key, ScancodeSet1>> = Mutex::new(
    Keyboard::new(ScancodeSet1::new(), Us104Key, HandleControl::Ignore),
);

pub fn init_idt() -> InterruptDescriptorTable {
    let mut idt = InterruptDescriptorTable::new();
    idt.breakpoint.set_handler_fn(breakpoint_handler);
    idt[Interrupts::Timer as usize].set_handler_fn(timer_handler);
    idt[Interrupts::Keyboard as usize].set_handler_fn(keyboard_handler);

    idt
}

pub static PICS: Mutex<ChainedPics> = Mutex::new(unsafe { ChainedPics::new(32, 40) });

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {}

extern "x86-interrupt" fn timer_handler(stack_frame: InterruptStackFrame) {
    // println!("tick");

    unsafe {
        PICS.lock().notify_end_of_interrupt(Interrupts::Timer as u8);
    }
}

extern "x86-interrupt" fn keyboard_handler(_stack_frame: InterruptStackFrame) {
    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };
    let mut kb = KEYBOARD.lock();
    let key_event = kb.add_byte(scancode);

    if let Ok(Some(key_event)) = key_event {
        if let Some(key) = kb.process_keyevent(key_event) {
            kbhandler::handle(key)
        }
    }

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(Interrupts::Keyboard as u8);
    }
}
