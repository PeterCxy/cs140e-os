use pi::timer;
use pi::interrupt::{Controller, Interrupt};

use console::kprintln;
use traps::TrapFrame;
use process::TICK;

pub fn handle_irq(interrupt: Interrupt, tf: &mut TrapFrame) {
    if interrupt == Interrupt::Timer1 {
        kprintln!("Timer interrupt!"); // TODO: Remove this
        timer::tick_in(TICK);
    }
}
