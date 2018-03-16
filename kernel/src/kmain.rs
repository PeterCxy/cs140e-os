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
    use pi::volatile::*;
    use pi::timer;

    let GPFSEL0 = unsafe {
        &mut *((IO_BASE + 0x200000) as *mut Volatile<u32>)
    };

    let GPSET0 = unsafe {
        &mut *((IO_BASE + 0x20001c) as *mut Volatile<u32>)
    };

    let GPCLR0 = unsafe {
        &mut *((IO_BASE + 0x200028) as *mut Volatile<u32>)
    };

    GPFSEL0.or_mask(1 << (4 % 10) * 3);

    loop {
        GPSET0.or_mask(1 << 4);
        timer::spin_sleep_ms(100);
        GPCLR0.or_mask(1 << 4);
        timer::spin_sleep_ms(100);
    }
}
