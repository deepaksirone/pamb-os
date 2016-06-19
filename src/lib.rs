#![feature(lang_items)]
#![no_std]
#![feature(const_fn)]
#![feature(unique)]
#![feature(asm)]

extern crate rlibc;
extern crate spin;
extern crate pic8259_simple;
extern crate cpuio;
#[macro_use]
extern crate lazy_static;
extern crate x86;
extern crate bit_field;

#[macro_use]
mod vga_buffer;
mod interrupts;

#[no_mangle]
pub extern fn rust_main()
{
    vga_buffer::clear_screen();
    use interrupts::PICS;
    use cpuio::Port;
    
    unsafe {
        asm!("cli");
        PICS.lock().initialize();
        interrupts::init();
        let mut pic_master_data: Port<u8> = Port::new(0x21);
        let mut time_w: Port<u8> = Port::new(0x80);
        let mut pic_slave_data: Port<u8> = Port::new(0xA1);
        pic_master_data.write(0xff);
        time_w.write(0);
        pic_slave_data.write(0xff);
        time_w.write(0);
        pic_master_data.write(!0x2);
        time_w.write(0); 
        asm!("sti");
        println!("DATA mask: {}", pic_master_data.read());

    }
    println!("Hello World!");
  
    loop { }
}

#[lang = "eh_personality"] extern fn eh_personality() {}
#[lang = "panic_fmt"] extern fn panic_fmt() -> !{ loop {} }


