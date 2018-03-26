use common::{IO_BASE, is_bit_set};
use volatile::prelude::*;
use volatile::{WriteVolatile, ReadVolatile};

const INT_BASE: usize = IO_BASE + 0xB000 + 0x200;

#[derive(Copy, Clone, PartialEq)]
pub enum Interrupt {
    Timer1 = 1,
    Timer3 = 3,
    Usb = 9,
    Gpio0 = 49,
    Gpio1 = 50,
    Gpio2 = 51,
    Gpio3 = 52,
    Uart = 57,
}

#[repr(C)]
#[allow(non_snake_case)]
struct Registers {
    IRQ_BASIC_PENDING: ReadVolatile<u32>,
    IRQ_PENDING: [ReadVolatile<u32>; 2],
    FIQ_CONTROL: WriteVolatile<u32>,
    ENABLE_IRQ: [WriteVolatile<u32>; 2],
    ENABLE_BASIC_IRQ: WriteVolatile<u32>,
    DISABLE_IRQ: [WriteVolatile<u32>; 2],
    DISABLE_BASIC_IRQ: WriteVolatile<u32>
}

/// An interrupt controller. Used to enable and disable interrupts as well as to
/// check if an interrupt is pending.
pub struct Controller {
    registers: &'static mut Registers
}

impl Controller {
    /// Returns a new handle to the interrupt controller.
    pub fn new() -> Controller {
        Controller {
            registers: unsafe { &mut *(INT_BASE as *mut Registers) },
        }
    }

    #[inline(always)]
    fn interrupt_resgiter_pos(int: Interrupt) -> (usize, u8) {
        let id = int as u64;
        let register_num = id / 32;
        let offset = id % 32;
        (register_num as usize, offset as u8)
    }

    /// Enables the interrupt `int`.
    pub fn enable(&mut self, int: Interrupt) {
        let (register_num, offset) = Self::interrupt_resgiter_pos(int);
        self.registers.ENABLE_IRQ[register_num].write(1 << offset);
    }

    /// Disables the interrupt `int`.
    pub fn disable(&mut self, int: Interrupt) {
        let (register_num, offset) = Self::interrupt_resgiter_pos(int);
        self.registers.DISABLE_IRQ[register_num].write(1 << offset);
    }

    /// Returns `true` if `int` is pending. Otherwise, returns `false`.
    pub fn is_pending(&self, int: Interrupt) -> bool {
        let (register_num, offset) = Self::interrupt_resgiter_pos(int);
        let register = self.registers.IRQ_PENDING[register_num].read();
        is_bit_set!(register, offset)
    }
}
