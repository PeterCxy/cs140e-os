#![feature(lang_items)]
#![feature(core_intrinsics)]
#![feature(const_fn)]
#![feature(asm)]
#![feature(optin_builtin_traits)]
#![feature(decl_macro)]
#![feature(repr_align)]
#![feature(attr_literals)]
#![feature(never_type)]
#![feature(ptr_internals)]

extern crate pi;
extern crate stack_vec;

pub mod lang_items;
pub mod mutex;
pub mod console;
pub mod shell;

#[no_mangle]
pub extern "C" fn kmain() {
    use pi::uart;
    use std::io::Write;
    // FIXME: Start the shell.
    
    // FIXME: TEST Code
    // REMOVE THIS.
    let mut buf = [0u8; 255];
    let mut i = 0;
    let mut uart_port = uart::MiniUart::new();
    uart_port.write("--> Welcome to Peter Kernel!\n\r".as_bytes()).unwrap();
    loop {
        let byte = uart_port.read_byte();

        if byte == '\r' as u8 {
            uart_port.write("\n\r> Received: ".as_bytes()).unwrap();
            uart_port.write(&buf).unwrap();
            uart_port.write("\n\r".as_bytes());
            buf = [0u8; 255];
            i = 0;
            continue;
        }

        uart_port.write_byte(byte);

        if i >= buf.len() {
            i = 0;
            buf = [0u8; 255];
        }

        buf[i] = byte;
        i += 1;
    }
}
