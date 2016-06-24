use ::vga_buffer::*;
use cpuio::Port;
use ::interrupts::*;

static KEY_CODE_TO_ASCII: [u8; 59] = *b"??1234567890-=??qwertyuiop[]\n?asdfghjkl;'`?\\zxcvbnm,./?*? ?";

#[naked]
pub unsafe extern "C" fn kbd_handler () {

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
