#![feature(asm, lang_items)]

extern crate xmodem;
extern crate pi;

pub mod lang_items;

/// Start address of the binary to load and of the bootloader.
const BINARY_START_ADDR: usize = 0x80000;
const BOOTLOADER_START_ADDR: usize = 0x4000000;

/// Pointer to where the loaded binary expects to be laoded.
const BINARY_START: *mut u8 = BINARY_START_ADDR as *mut u8;

/// Free space between the bootloader and the loaded binary's start address.
const MAX_BINARY_SIZE: usize = BOOTLOADER_START_ADDR - BINARY_START_ADDR;

/// Branches to the address `addr` unconditionally.
fn jump_to(addr: *mut u8) -> ! {
    unsafe {
        asm!("br $0" : : "r"(addr as usize));
        loop { asm!("nop" :::: "volatile")  }
    }
}

#[no_mangle]
pub extern "C" fn kmain() {
    loop {
        try_load_kernel();
        pi::timer::spin_sleep_ms(500);
    }
}

// Try to load the kernel from UART using Xmodem protocol
fn try_load_kernel() {
    let buf = unsafe {
        std::slice::from_raw_parts_mut(BINARY_START, MAX_BINARY_SIZE)
    };

    // Transmit using Xmodem over UART port
    let mut uart = pi::uart::MiniUart::new();
    uart.set_read_timeout(1000);
    match xmodem::Xmodem::receive(uart, buf) {
        Ok(_) => jump_to(BINARY_START),
        Err(_) => ()
    }
}