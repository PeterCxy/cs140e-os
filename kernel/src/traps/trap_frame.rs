#[repr(C)]
#[derive(Default, Debug, Copy, Clone)]
pub struct TrapFrame {
    pub stack_pointer: u64,   // SP_ELs
    pub thread_id: u64,        // TPIDR_ELs
    pub program_state: u64,   // SPSR_ELx
    pub program_counter: u64, // ELR_ELx
    pub floating_point_resgiters: [u128; 32], // Order: TODO
    pub general_resgiters: [u64; 32] // Order: TODO (general_resgiters[30] is "reserved" field)
}
