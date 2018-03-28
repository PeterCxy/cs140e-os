use traps::TrapFrame;
use process::{State, Stack};
use process::state::EventPollFn;
use std::mem;

fn process_state_poll_nop(_process: &mut Process) -> bool {
    false
}

/// Type alias for the type of a process ID.
pub type Id = u64;

/// A structure that represents the complete state of a process.
#[derive(Debug)]
pub struct Process {
    /// The saved trap frame of a process.
    pub trap_frame: Box<TrapFrame>,
    /// The memory allocation used for the process's stack.
    pub stack: Stack,
    /// The scheduling state of the process.
    pub state: State,
}

impl Process {
    /// Creates a new process with a zeroed `TrapFrame` (the default), a zeroed
    /// stack of the default size, and a state of `Ready`.
    ///
    /// If enough memory could not be allocated to start the process, returns
    /// `None`. Otherwise returns `Some` of the new `Process`.
    pub fn new() -> Option<Process> {
        Stack::new()
            .map(|stack| {
                Process {
                    trap_frame: Box::new(TrapFrame {
                        stack_pointer: 0,
                        thread_id: 0,
                        program_state: 0,
                        program_counter: 0,
                        floating_point_registers: [0; 32],
                        general_registers: [0; 32]
                    }),
                    stack,
                    state: State::Ready
                }
            })
    }

    // Create process with a given entry point address
    pub fn create_process(entry: *const ()) -> Option<Process> {
        Self::new()
            .map(|mut process| {
                let sp = unsafe {
                    process.stack.top().as_u64()
                };
                process.trap_frame.stack_pointer = sp;
                process.trap_frame.program_counter = entry as u64;
                process
            })
    }

    /// Returns `true` if this process is ready to be scheduled.
    ///
    /// This functions returns `true` only if one of the following holds:
    ///
    ///   * The state is currently `Ready`.
    ///
    ///   * An event being waited for has arrived.
    ///
    ///     If the process is currently waiting, the corresponding event
    ///     function is polled to determine if the event being waiting for has
    ///     occured. If it has, the state is switched to `Ready` and this
    ///     function returns `true`.
    ///
    /// Returns `false` in all other cases.
    pub fn is_ready(&mut self) -> bool {
        let mut poll_fn: EventPollFn = Box::new(process_state_poll_nop);
        match self.state {
            State::Ready => return true,
            State::Running => return false,
            State::Waiting(ref mut f) => {
                mem::swap(f, &mut poll_fn);
            }
        }

        let ret = poll_fn(self);
        if let State::Waiting(ref mut f) = self.state {
            mem::swap(f, &mut poll_fn);
        }
        return ret;
    }
}
