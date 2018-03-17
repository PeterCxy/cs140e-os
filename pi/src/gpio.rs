use core::marker::PhantomData;

use common::{IO_BASE, states, is_bit_set};
use volatile::prelude::*;
use volatile::{Volatile, WriteVolatile, ReadVolatile, Reserved};

/// An alternative GPIO function.
#[repr(u8)]
pub enum Function {
    Input = 0b000,
    Output = 0b001,
    Alt0 = 0b100,
    Alt1 = 0b101,
    Alt2 = 0b110,
    Alt3 = 0b111,
    Alt4 = 0b011,
    Alt5 = 0b010
}

#[repr(C)]
#[allow(non_snake_case)]
struct Registers {
    FSEL: [Volatile<u32>; 6],
    __r0: Reserved<u32>,
    SET: [WriteVolatile<u32>; 2],
    __r1: Reserved<u32>,
    CLR: [WriteVolatile<u32>; 2],
    __r2: Reserved<u32>,
    LEV: [ReadVolatile<u32>; 2],
    __r3: Reserved<u32>,
    EDS: [Volatile<u32>; 2],
    __r4: Reserved<u32>,
    REN: [Volatile<u32>; 2],
    __r5: Reserved<u32>,
    FEN: [Volatile<u32>; 2],
    __r6: Reserved<u32>,
    HEN: [Volatile<u32>; 2],
    __r7: Reserved<u32>,
    LEN: [Volatile<u32>; 2],
    __r8: Reserved<u32>,
    AREN: [Volatile<u32>; 2],
    __r9: Reserved<u32>,
    AFEN: [Volatile<u32>; 2],
    __r10: Reserved<u32>,
    PUD: Volatile<u32>,
    PUDCLK: [Volatile<u32>; 2],
}

/*
 * Calculate which of the GPFSEL registers
 * control the state of `pin`
 */
#[inline(always)]
fn fsel_index(pin: u8) -> usize {
    (pin / 10) as usize
}

/*
 * Calculate which 3 bits of the GPFSEL register
 * control the state of `pin`
 */
#[inline(always)]
fn fsel_offset(pin: u8) -> u8 {
    (pin % 10) * 3
}

#[inline(always)]
fn fsel_where(pin: u8) -> (usize, u8) {
    (fsel_index(pin), fsel_offset(pin))
}

/*
 * Calculate which GPSET / GPCLR / GPLEV
 * correspond to `pin`
 */
#[inline(always)]
fn gpio_index(pin: u8) -> usize {
    (pin / 32) as usize
}

/*
 * Calculate which bit in the GPSET / GPCLR / GPLEV
 * register correspond to `pin`
 */
#[inline(always)]
fn gpio_offset(pin: u8) -> u8 {
    pin % 32
}

#[inline(always)]
fn gpio_where(pin: u8) -> (usize, u8) {
    (gpio_index(pin), gpio_offset(pin))
}

/// Possible states for a GPIO pin.
states! {
    Uninitialized, Input, Output, Alt
}

/// A GPIO pin in state `State`.
///
/// The `State` generic always corresponds to an uninstantiatable type that is
/// use solely to mark and track the state of a given GPIO pin. A `Gpio`
/// structure starts in the `Uninitialized` state and must be transitions into
/// one of `Input`, `Output`, or `Alt` via the `into_input`, `into_output`, and
/// `into_alt` methods before it can be used.
pub struct Gpio<State> {
    pin: u8,
    registers: &'static mut Registers,
    _state: PhantomData<State>
}

/// The base address of the `GPIO` registers.
const GPIO_BASE: usize = IO_BASE + 0x200000;

impl<T> Gpio<T> {
    /// Transitions `self` to state `S`, consuming `self` and returning a new
    /// `Gpio` instance in state `S`. This method should _never_ be exposed to
    /// the public!
    #[inline(always)]
    fn transition<S>(self) -> Gpio<S> {
        Gpio {
            pin: self.pin,
            registers: self.registers,
            _state: PhantomData
        }
    }
}

impl Gpio<Uninitialized> {
    /// Returns a new `GPIO` structure for pin number `pin`.
    ///
    /// # Panics
    ///
    /// Panics if `pin` > `53`.
    pub fn new(pin: u8) -> Gpio<Uninitialized> {
        if pin > 53 {
            panic!("Gpio::new(): pin {} exceeds maximum of 53", pin);
        }

        Gpio {
            registers: unsafe { &mut *(GPIO_BASE as *mut Registers) },
            pin: pin,
            _state: PhantomData
        }
    }

    /// Enables the alternative function `function` for `self`. Consumes self
    /// and returns a `Gpio` structure in the `Alt` state.
    pub fn into_alt(self, function: Function) -> Gpio<Alt> {
        let (index, offset) = fsel_where(self.pin);
        self.registers.FSEL[index].or_mask((function as u32) << offset);
        self.transition()
    }

    /// Sets this pin to be an _output_ pin. Consumes self and returns a `Gpio`
    /// structure in the `Output` state.
    pub fn into_output(self) -> Gpio<Output> {
        self.into_alt(Function::Output).transition()
    }

    /// Sets this pin to be an _input_ pin. Consumes self and returns a `Gpio`
    /// structure in the `Input` state.
    pub fn into_input(self) -> Gpio<Input> {
        self.into_alt(Function::Input).transition()
    }
}

impl Gpio<Output> {
    /// Sets (turns on) the pin.
    pub fn set(&mut self) {
        let (index, offset) = gpio_where(self.pin);
        self.registers.SET[index].write(1 << offset);
    }

    /// Clears (turns off) the pin.
    pub fn clear(&mut self) {
        let (index, offset) = gpio_where(self.pin);
        self.registers.CLR[index].write(1 << offset);
    }
}

impl Gpio<Input> {
    /// Reads the pin's value. Returns `true` if the level is high and `false`
    /// if the level is low.
    pub fn level(&mut self) -> bool {
        let (index, offset) = gpio_where(self.pin);
        let lev = self.registers.LEV[index].read();
        is_bit_set!(lev, offset)
    }
}
