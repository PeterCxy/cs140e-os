#![feature(lang_items)]
#![feature(core_intrinsics)]
#![feature(const_fn)]
#![feature(asm)]
#![feature(optin_builtin_traits)]
#![feature(decl_macro)]
#![feature(repr_align)]
#![feature(attr_literals)]
#![feature(exclusive_range_pattern)]
#![feature(i128_type)]
#![feature(never_type)]
#![feature(unique)]
#![feature(pointer_methods)]
#![feature(naked_functions)]
#![feature(fn_must_use)]
#![feature(alloc, allocator_api, global_allocator)]
#![feature(ptr_internals)]

#[macro_use]
#[allow(unused_imports)]
extern crate alloc;
extern crate pi;
extern crate stack_vec;
extern crate fat32;

pub mod allocator;
pub mod lang_items;
pub mod mutex;
pub mod console;
pub mod shell;
pub mod fs;
pub mod traps;
pub mod aarch64;
pub mod process;
pub mod vm;

#[cfg(not(test))]
use allocator::Allocator;
use fs::FileSystem;
use process::GlobalScheduler;

#[cfg(not(test))]
#[global_allocator]
pub static ALLOCATOR: Allocator = Allocator::uninitialized();

pub static FILE_SYSTEM: FileSystem = FileSystem::uninitialized();

pub static SCHEDULER: GlobalScheduler = GlobalScheduler::uninitialized();

#[no_mangle]
#[cfg(not(test))]
pub extern "C" fn kmain() {
    // Initialize the memory alloator
    ALLOCATOR.initialize();

    // Simulate some long-running initialization task
    // because otherwise we won't have time to connect to
    // the UART port to debug
    pi::timer::spin_sleep_ms(1000);
    
    // Initialze file system
    FILE_SYSTEM.initialize();

    // Start scheduler
    SCHEDULER.start();
}

#[no_mangle]
#[cfg(not(test))]
pub extern "C" fn start_shell() {
    process::Process::create_process(start_test_process as *const ())
        .map(|p| SCHEDULER.add(p)).unwrap();
    process::Process::create_process(start_test_process_2 as *const ())
        .map(|p| SCHEDULER.add(p)).unwrap();
    console::kprintln!("Hello world from user space!");

    loop {
        shell::shell("$ ");
    }
}

#[no_mangle]
#[cfg(not(test))]
pub extern "C" fn start_test_process() {
    let mut i = 0;
    loop {
        pi::timer::spin_sleep_ms(200);
        console::kprintln!("test {}", i);
        i += 1;
    }
}

#[no_mangle]
#[cfg(not(test))]
pub extern "C" fn start_test_process_2() {
    let mut i = 0;
    loop {
        pi::timer::spin_sleep_ms(500);
        console::kprintln!("test2 {}", i);
        i += 1;
    }
}