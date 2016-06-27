#![feature(lang_items)]
#![no_std]
#![feature(const_fn)]
#![feature(unique)]
#![feature(asm)]
#![feature(naked_functions)]
extern crate rlibc;
extern crate spin;
extern crate pic8259_simple;
extern crate cpuio;
#[macro_use]
extern crate lazy_static;
extern crate x86;
extern crate multiboot2;
extern crate bit_field;

#[macro_use]
mod vga_buffer;
mod interrupts;

#[no_mangle]
pub extern fn rust_main(multiboot_information_address: usize)
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

        let boot_info = unsafe{ multiboot2::load(multiboot_information_address) };
        let memory_map_tag = boot_info.memory_map_tag().expect("Memory map tag required");

    
        println!("Memory Areas:");
        for area in memory_map_tag.memory_areas() {
            println!("  start: 0x{:x}, length: 0x{:x}",
                     area.base_addr, area.length);
        }

        let elf_sections_tag = boot_info.elf_sections_tag().expect("Elf-sections tag required");
        println!("Kernel Sections:");
        for section in elf_sections_tag.sections() {
            println!("   addr: 0x{:x}, size: 0x{:x}, flags: 0x{:x}"
                     , section.addr, section.size, section.flags);
        }
        let kernel_start = elf_sections_tag.sections().map(|s| s.addr).min().unwrap();
        let kernel_end = elf_sections_tag.sections().map(|s| s.addr + s.size).max().unwrap();

        let multiboot_start = multiboot_information_address;
        let multiboot_end = multiboot_start + (boot_info.total_size as usize); 

        println!("kernel start: {} kernel end: {}", kernel_start, kernel_end);
        println!("multiboot_start: {} multiboot_end: {}", multiboot_start, multiboot_end);
    }
    println!("Hello World!");
  
    loop { }
}

#[lang = "eh_personality"] extern fn eh_personality() {}
#[lang = "panic_fmt"]
extern fn panic_fmt(fmt: core::fmt::Arguments, file: &str, line: u32) -> !
{ 
    println!("\n\nPANIC in {} at line {}:", file, line);
    println!("   {}", fmt);
    loop {}
}


