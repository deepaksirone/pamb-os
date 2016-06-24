use pic8259_simple::ChainedPics;
use spin::Mutex;
use cpuio::Port;

// Map PIC interrupts to 0x20 through 0x2f.
mod idt;
//use ::vga_buffer::*;
#[macro_use]
#[macro_escape]
mod generic;
mod keyboard;
pub static PICS: Mutex<ChainedPics> = Mutex::new(unsafe { ChainedPics::new(0x20, 0x28) });

lazy_static! {
    static ref IDT: idt::Idt = {
        let mut idt = idt::Idt::new();
        idt.set_handler(33, keyboard::kbd_handler);
        idt
    };
}

pub fn init() {
    IDT.load();
}
