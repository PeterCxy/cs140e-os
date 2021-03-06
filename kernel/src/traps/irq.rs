use pi::timer;
use pi::interrupt::{Controller, Interrupt};

use console::kprintln;
use traps::TrapFrame;
use process::{TICK, State};
use SCHEDULER;

pub fn handle_irq(interrupt: Interrupt, tf: &mut TrapFrame) {
    if interrupt == Interrupt::Timer1 {
        SCHEDULER.switch(State::Ready, tf).expect("Fatal: no process running");
        timer::tick_in(TICK);
    }
}
