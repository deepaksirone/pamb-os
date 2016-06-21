use pic8259_simple::ChainedPics;
use spin::Mutex;

// Map PIC interrupts to 0x20 through 0x2f.
mod idt;

pub static PICS: Mutex<ChainedPics> = Mutex::new(unsafe { ChainedPics::new(0x20, 0x28) });

lazy_static! {
    static ref IDT: idt::Idt = {
        let mut idt = idt::Idt::new();
        idt.set_handler(33, kbd_handler);
        idt
    }; 
}

#[naked]
unsafe extern "C" fn kbd_handler () {

//        asm!("mov rax, rbx"::::"intel");
      asm!(concat!(
 //       "push rbp",                    "\n\t",
        
//        "pusha",                       "\n\t",
//        "cld",                          "\n\t",
        
        "push rbp",                    "\n\t",
        "push r15",                    "\n\t",
        "push r14",                    "\n\t",
        "push r13",                    "\n\t",
        "push r12",                    "\n\t",
        "push r11",                    "\n\t",
        "push r10",                    "\n\t",
        "push r9",                    "\n\t",
        "push r8",                    "\n\t",
        "push rsi",                    "\n\t",
        "push rdi",                    "\n\t",
        "push rdx",                    "\n\t",
        "push rcx",                    "\n\t",
        "push rbx",                    "\n\t",
        "push rax",                    "\n\t",
//        "push rbp",                    "\n\t",
        
        "call  keyboard_handler",                    "\n\t",

//        "popa",                        "\n\t",
        "pop rax",                    "\n\t",
        "pop rbx",                    "\n\t",
        "pop rcx",                    "\n\t",
        "pop rdx",                    "\n\t",
        "pop rdi",                    "\n\t",
        "pop rsi",                     "\n\t",
        "pop r8",                    "\n\t",
        "pop r9",                    "\n\t",
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
#[no_mangle] 
extern "C" fn keyboard_handler() {

    println!("Key pressed");
    unsafe {
    asm!(concat!(
        "in al, 0x60",          "\n\t",
        "mov al, 0x20",             "\n\t",
        "out 0x20, al",            "\n\t",
        "mov al, 0", "\n\t",
        "out 0x80, al", "\n\t",
        "mov al, 0x20", "\n\t") :::: "volatile", "intel");
    }
}

pub fn init() {
    IDT.load();
}


