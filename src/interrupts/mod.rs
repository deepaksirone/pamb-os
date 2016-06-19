use pic8259_simple::ChainedPics;
use spin::Mutex;

// Map PIC interrupts to 0x20 through 0x2f.
mod idt;

pub static PICS: Mutex<ChainedPics> = Mutex::new(unsafe { ChainedPics::new(0x20, 0x28) });

lazy_static! {
    static ref IDT: idt::Idt = {
        let mut idt = idt::Idt::new();
        idt.set_handler(33, keyboard_handler);
        idt
    }; 
}

extern "C" fn keyboard_handler() -> ! {
    println!("Key pressed");
    loop {} 
}

pub fn init() {
    IDT.load();
}


