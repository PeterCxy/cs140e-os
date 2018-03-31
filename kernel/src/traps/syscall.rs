use traps::TrapFrame;
use console::kprintln;
use SCHEDULER;
use pi::timer;
use process;

/// Sleep for `ms` milliseconds.
///
/// This system call takes one parameter: the number of milliseconds to sleep.
///
/// In addition to the usual status value, this system call returns one
/// parameter: the approximate true elapsed time from when `sleep` was called to
/// when `sleep` returned.
pub fn sleep(ms: u32, tf: &mut TrapFrame) {
    let start_time = timer::current_time();
    let f = Box::new(move |p: &mut process::Process| {
        let elapsed_time = timer::current_time() - start_time;
        if elapsed_time >= (ms as u64) * 1000 {
            // Return the actual elapsed time from this syscall via x0
            p.trap_frame.general_registers[31] = elapsed_time / 1000;
            return true;
        } else {
            return false;
        }
    });
    SCHEDULER.switch(process::State::Waiting(f), tf).unwrap();
}

// TODO: Implement error return
pub fn handle_syscall(num: u16, tf: &mut TrapFrame) {
    if num == 1 {
        // Pass x0 as the sleeping time for syscall 1: sleep
        sleep(tf.general_registers[31] as u32, tf);
    } else {
        unimplemented!();
    }
}

pub fn call_sleep(ms: u32) -> u32 {
    let ret: u32;
    unsafe {
        asm!("mov x0, $1
              svc 1
              mov $0, x0"
              : "=r"(ret) : "r"(ms) : "x0" : "volatile"
        );
    }
    return ret;
}