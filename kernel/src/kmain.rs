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
    // FIXME: Start the shell.
    
    // FIXME: TEST Blinky Code
    // REMOVE THIS.
    use pi::common::IO_BASE;
    use pi::gpio;
    use pi::timer;

    let mut led = gpio::Gpio::new(4).into_output();
    let mut button = gpio::Gpio::new(17).into_input();

    loop {
        if button.level() {
            continue;
        }

        led.set();
        timer::spin_sleep_ms(500);
        led.clear();
        timer::spin_sleep_ms(500);
    }
}
