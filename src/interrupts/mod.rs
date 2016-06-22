use pic8259_simple::ChainedPics;
use spin::Mutex;
use cpuio::Port;

// Map PIC interrupts to 0x20 through 0x2f.
mod idt;
use ::vga_buffer::*;
pub static PICS: Mutex<ChainedPics> = Mutex::new(unsafe { ChainedPics::new(0x20, 0x28) });

lazy_static! {
    static ref IDT: idt::Idt = {
        let mut idt = idt::Idt::new();
        idt.set_handler(33, kbd_handler);
        idt
    };
}

static KEY_CODE_TO_ASCII: [u8; 59] = *b"??1234567890-=??qwertyuiop[]\n?asdfghjkl;'`?\\zxcvbnm,./?*? ?";

macro_rules! prologue {
        () => {
                asm!(concat!(
                "push rbp",                    "\n\t",
                "push r15",                    "\n\t",
                "push r14",                    "\n\t",
                "push r13",                    "\n\t",
                "push r12",                    "\n\t",
                "push r11",                    "\n\t",
                "push r10",                    "\n\t",
                "push r9",                     "\n\t",
                "push r8",                     "\n\t",
                "push rsi",                    "\n\t",
                "push rdi",                    "\n\t",
                "push rdx",                    "\n\t",
                "push rcx",                    "\n\t",
                "push rbx",                    "\n\t",
                "push rax",                    "\n\t")
                  :::: "volatile", "intel");
            }
}

macro_rules! epilogue {

        () => {
                 asm!(concat!(
                "pop rax",                    "\n\t",
                "pop rbx",                    "\n\t",
                "pop rcx",                    "\n\t",
                "pop rdx",                    "\n\t",
                "pop rdi",                    "\n\t",
                "pop rsi",                    "\n\t",
                "pop r8",                     "\n\t",
                "pop r9",                     "\n\t",
                "pop r10",                    "\n\t",
                "pop r11",                    "\n\t",
                "pop r12",                    "\n\t",
                "pop r13",                    "\n\t",
                "pop r14",                    "\n\t",
                "pop r15",                    "\n\t",
                "pop rbp",                    "\n\t",


                "iretq",                       "\n\t")
                   :::: "volatile", "intel");
            }

}

#[naked]
unsafe extern "C" fn kbd_handler () {

      prologue!();
      asm!("call  keyboard_handler":::: "volatile", "intel");
      epilogue!();
}
#[no_mangle]
unsafe extern "C" fn keyboard_handler() {
//    println!("Key pressed");
    let mut kbd_data_port: Port<u8> = Port::new(0x60);
    let scan_code = kbd_data_port.read();
    match KEY_CODE_TO_ASCII.get(scan_code as usize) {
              Some(ascii) => WRITER.lock().write_byte(*ascii),
                    None => ()
    }
    PICS.lock().notify_end_of_interrupt(0x21);
}

pub fn init() {
    IDT.load();
}
