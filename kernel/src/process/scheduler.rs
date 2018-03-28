use std::collections::VecDeque;
use std::mem;

use aarch64;
use console;
use mutex::Mutex;
use process::{Process, State, Id};
use traps::TrapFrame;
use start_shell;

use pi::{timer, interrupt};

/// The `tick` time.
// FIXME: When you're ready, change this to something more reasonable.
pub const TICK: u32 = 2 * 1000 * 1000;

/// Process scheduler for the entire machine.
#[derive(Debug)]
pub struct GlobalScheduler(Mutex<Option<Scheduler>>);

impl GlobalScheduler {
    /// Returns an uninitialized wrapper around a local scheduler.
    pub const fn uninitialized() -> GlobalScheduler {
        GlobalScheduler(Mutex::new(None))
    }

    /// Adds a process to the scheduler's queue and returns that process's ID.
    /// For more details, see the documentation on `Scheduler::add()`.
    pub fn add(&self, process: Process) -> Option<Id> {
        self.0.lock().as_mut().expect("scheduler uninitialized").add(process)
    }

    /// Performs a context switch using `tf` by setting the state of the current
    /// process to `new_state`, saving `tf` into the current process, and
    /// restoring the next process's trap frame into `tf`. For more details, see
    /// the documentation on `Scheduler::switch()`.
    #[must_use]
    pub fn switch(&self, new_state: State, tf: &mut TrapFrame) -> Option<Id> {
        self.0.lock().as_mut().expect("scheduler uninitialized").switch(new_state, tf)
    }

    /// Initializes the scheduler and starts executing processes in user space
    /// using timer interrupt based preemptive scheduling. This method should
    /// not return under normal conditions.
    pub fn start(&self) {
        *self.0.lock() = Some(Scheduler::new());
        interrupt::Controller::new().enable(interrupt::Interrupt::Timer1);
        timer::tick_in(TICK);

        // Bootstrap the first process (init process)
        Process::create_process(start_shell as *const ())
            .map(|mut shell_process| {
                // Clone the trap frame because we have to
                // pass a copy to context_restore
                let mut trap_frame_clone = shell_process.trap_frame.clone();
                let id = self.add(shell_process).unwrap();

                // Bootstrap the thread_id in the cloned trap frame
                trap_frame_clone.thread_id = id as u64;

                // We don't need to set SPSR because 0 means switching to EL0
                // and unmasking all the necessary exceptions
                // Call `context_restore` in `init.S` to switch to EL0
                unsafe {
                    asm!("mov x0, $0
                          mov x1, #1
                          bl context_restore"
                        :: "r"(&*trap_frame_clone) :: "volatile");
                }
            }).expect("WTF");
    }
}

#[derive(Debug)]
struct Scheduler {
    processes: VecDeque<Process>,
    current: Option<Id>,
    last_id: Option<Id>,
}

impl Scheduler {
    /// Returns a new `Scheduler` with an empty queue.
    fn new() -> Scheduler {
        Scheduler {
            processes: VecDeque::new(),
            current: None,
            last_id: Some(0)
        }
    }

    /// Adds a process to the scheduler's queue and returns that process's ID if
    /// a new process can be scheduled. The process ID is newly allocated for
    /// the process and saved in its `trap_frame`. If no further processes can
    /// be scheduled, returns `None`.
    ///
    /// If this is the first process added, it is marked as the current process.
    /// It is the caller's responsibility to ensure that the first time `switch`
    /// is called, that process is executing on the CPU.
    fn add(&mut self, mut process: Process) -> Option<Id> {
        if self.last_id.is_none() {
            return None;
        }

        let last_id = self.last_id.unwrap() + 1;
        process.trap_frame.thread_id = last_id;
        self.processes.push_back(process);

        if last_id == ::std::u64::MAX {
            // TODO: implement wrapping for process IDs
            self.last_id = None;
        } else {
            self.last_id = Some(last_id);
        }

        if self.processes.len() == 1 {
            self.current = Some(last_id);
        }

        return Some(last_id);
    }

    /// Sets the current process's state to `new_state`, finds the next process
    /// to switch to, and performs the context switch on `tf` by saving `tf`
    /// into the current process and restoring the next process's trap frame
    /// into `tf`. If there is no current process, returns `None`. Otherwise,
    /// returns `Some` of the process ID that was context switched into `tf`.
    ///
    /// This method blocks until there is a process to switch to, conserving
    /// energy as much as possible in the interim.
    fn switch(&mut self, new_state: State, tf: &mut TrapFrame) -> Option<Id> {
        if self.current.is_none() {
            return None;
        }

        // Save link register for returning into HANDLER
        // The actual link register for EL0 is saved in PSTATE
        // `x30` here is for EL1
        let x30 = tf.general_registers[1];

        // Move the current process to the back of the queue
        let mut p = self.processes.pop_front().unwrap();
        p.state = new_state;
        mem::swap(tf, &mut *(p.trap_frame));
        self.processes.push_back(p);
        self.current = None;

        loop {
            // Find a ready process to execute
            for i in 0..self.processes.len() {
                if self.processes[i].is_ready() {
                    // Process is ready!
                    let mut process = self.processes.remove(i).unwrap();
                    let id = process.trap_frame.thread_id as Id;
                    process.state = State::Running;
                    // Keep `x30` (link register) from this exception
                    process.trap_frame.general_registers[1] = x30;

                    // Move its trap frame into `tf`
                    mem::swap(tf, &mut *(process.trap_frame));

                    // Move it to the front of the queue
                    self.processes.push_front(process);

                    // Mark it as current
                    self.current = Some(id);
                    return self.current.clone();
                }
            }

            aarch64::wait_for_interrupt();
        }
    }
}
