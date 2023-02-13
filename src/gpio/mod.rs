//! GPIO and Alternate function

use core::fmt;
use core::marker::PhantomData;

use crate::pac::{AFIO, EXTI};
use crate::rcc::{Enable, APB2};

pub use embedded_hal::digital::v2::PinState;

mod convert;
mod hal_02;
mod partially_erased;
pub use partially_erased::{PEPin, PartiallyErasedPin};

/// A filler pin type
#[derive(Debug)]
pub struct NoPin;

/// Extension trait to split a GPIO peripheral in independent pins and registers
pub trait GpioExt {
    /// The parts to split the GPIO into
    type Parts;

    /// Splits the GPIO block into independent pins and registers
    fn split(self) -> Self::Parts;
}

pub trait PinExt {
    type Mode;
    /// Return pin number
    fn pin_id(&self) -> u8;
    /// Return port number
    fn port_id(&self) -> u8;
}

/// Some alternate mode (type state)
pub struct Alternate<Otype = PushPull>(PhantomData<Otype>);

/// Input mode (type state)
pub struct Input<MODE = Floating> {
    _mode: PhantomData<MODE>,
}

/// Floating input (type state)
pub struct Floating;

/// Pulled down input (type state)
pub struct PullDown;

/// Pulled up input (type state)
pub struct PullUp;

/// Open drain input or output (type state)
pub struct OpenDrain;

/// Output mode (type state)
pub struct Output<MODE = PushPull> {
    _mode: PhantomData<MODE>,
}

/// Push pull output (type state)
pub struct PushPull;

/// Analog mode (type state)
pub struct Analog;

pub type Debugger = Alternate<PushPull>;

/// Slew rates available for Output and relevant AlternateMode Pins
pub enum Speed {
    /// Slew at 10Mhz
    Mhz10 = 0b01, // (yes, this one is "less" then 2Mhz)
    /// Slew at 2Mhz
    Mhz2 = 0b10,
    /// Slew at 50Mhz
    Mhz50 = 0b11,
}

/// Allow setting of the slew rate of an IO pin
///
/// Initially all pins are set to the maximum slew rate
pub trait OutputSpeed<CR> {
    fn set_speed(&mut self, cr: &mut CR, speed: Speed);
}

// TODO: interrupts
// Edge, Interruptable

/// Generic pin type
///
/// - `MODE` is one of the pin modes (see [Modes](crate::gpio#modes) section).
/// - `P` is port name: `A` for GPIOA, `B` for GPIOB, etc.
/// - `N` is pin number: from `0` to `15`.
pub struct Pin<const P: char, const N: u8, MODE = Input<Floating>> {
    _mode: PhantomData<MODE>,
}
impl<const P: char, const N: u8, MODE> Pin<P, N, MODE> {
    const fn new() -> Self {
        Self { _mode: PhantomData }
    }
}

impl<const P: char, const N: u8, MODE> fmt::Debug for Pin<P, N, MODE> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_fmt(format_args!(
            "P{}{}<{}>",
            P,
            N,
            crate::stripped_type_name::<MODE>()
        ))
    }
}

impl<const P: char, const N: u8, MODE> PinExt for Pin<P, N, MODE> {
    type Mode = MODE;

    #[inline(always)]
    fn pin_id(&self) -> u8 {
        N
    }
    #[inline(always)]
    fn port_id(&self) -> u8 {
        P as u8 - b'A'
    }
}

impl<const P: char, const N: u8, MODE> Pin<P, N, Output<MODE>> {
    /// Set pin speed
    pub fn set_speed(self, speed: Speed) -> Self {
        let offset = 4 * { N & 0b111 };

        if N >= 8 {
            unsafe {
                (*Gpio::<P>::ptr()).cfghr.modify(|r, w| {
                    w.bits((r.bits() & !(0b11 << offset)) | ((speed as u32) << offset))
                })
            };
        } else {
            unsafe {
                (*Gpio::<P>::ptr()).cfglr.modify(|r, w| {
                    w.bits((r.bits() & !(0b11 << offset)) | ((speed as u32) << offset))
                })
            };
        }

        self
    }
}

// NOTE: No internal_pull_up and internal_pull_down for Output<OpenDrain>

impl<const P: char, const N: u8> Pin<P, N, Alternate<PushPull>> {
    /// Set pin speed
    pub fn set_speed(self, speed: Speed) -> Self {
        let offset = 4 * { N & 0b111 };

        if N >= 8 {
            unsafe {
                (*Gpio::<P>::ptr()).cfghr.modify(|r, w| {
                    w.bits((r.bits() & !(0b11 << offset)) | ((speed as u32) << offset))
                })
            };
        } else {
            unsafe {
                (*Gpio::<P>::ptr()).cfglr.modify(|r, w| {
                    w.bits((r.bits() & !(0b11 << offset)) | ((speed as u32) << offset))
                })
            };
        }

        self
    }
}

impl<const P: char, const N: u8> Pin<P, N, Alternate<PushPull>> {
    /// Turns pin alternate configuration pin into open drain
    pub fn set_open_drain(self) -> Pin<P, N, Alternate<OpenDrain>> {
        // CNFy
        let offset = 4 * { N & 0b111 } + 2;

        if N >= 8 {
            unsafe {
                (*Gpio::<P>::ptr())
                    .cfghr
                    .modify(|r, w| w.bits((r.bits() & !(0b11 << offset)) | (0b10 << offset)))
            };
        } else {
            unsafe {
                (*Gpio::<P>::ptr())
                    .cfglr
                    .modify(|r, w| w.bits((r.bits() & !(0b11 << offset)) | (0b10 << offset)))
            };
        }

        Pin::new()
    }
}

// TODO: Erase pin number, Erase pin number and port number

impl<const P: char, const N: u8, MODE> Pin<P, N, MODE> {
    /// Set the output of the pin regardless of its mode.
    /// Primarily used to set the output value of the pin
    /// before changing its mode to an output to avoid
    /// a short spike of an incorrect value
    #[inline(always)]
    fn _set_state(&mut self, state: PinState) {
        match state {
            PinState::High => self._set_high(),
            PinState::Low => self._set_low(),
        }
    }
    #[inline(always)]
    fn _set_high(&mut self) {
        // NOTE(unsafe) atomic write to a stateless register
        unsafe { (*Gpio::<P>::ptr()).bshr.write(|w| w.bits(1 << N)) }
    }
    #[inline(always)]
    fn _set_low(&mut self) {
        // NOTE(unsafe) atomic write to a stateless register
        unsafe { (*Gpio::<P>::ptr()).bshr.write(|w| w.bits(1 << (16 + N))) }
    }
    #[inline(always)]
    fn _is_set_low(&self) -> bool {
        // NOTE(unsafe) atomic read with no side effects
        unsafe { (*Gpio::<P>::ptr()).outdr.read().bits() & (1 << N) == 0 }
    }
    #[inline(always)]
    fn _is_low(&self) -> bool {
        // NOTE(unsafe) atomic read with no side effects
        unsafe { (*Gpio::<P>::ptr()).indr.read().bits() & (1 << N) == 0 }
    }
}

impl<const P: char, const N: u8, MODE> Pin<P, N, Output<MODE>> {
    #[inline(always)]
    pub fn set_high(&mut self) {
        self._set_high()
    }

    #[inline(always)]
    pub fn set_low(&mut self) {
        self._set_low()
    }

    #[inline(always)]
    pub fn get_state(&self) -> PinState {
        if self.is_set_low() {
            PinState::Low
        } else {
            PinState::High
        }
    }

    #[inline(always)]
    pub fn set_state(&mut self, state: PinState) {
        match state {
            PinState::Low => self.set_low(),
            PinState::High => self.set_high(),
        }
    }

    #[inline(always)]
    pub fn is_set_high(&self) -> bool {
        !self.is_set_low()
    }

    #[inline(always)]
    pub fn is_set_low(&self) -> bool {
        self._is_set_low()
    }

    #[inline(always)]
    pub fn toggle(&mut self) {
        if self.is_set_low() {
            self.set_high()
        } else {
            self.set_low()
        }
    }
}

// Special high/low for open drain output

impl<const P: char, const N: u8> Pin<P, N, Output<OpenDrain>> {
    #[inline(always)]
    pub fn is_high(&self) -> bool {
        !self.is_low()
    }

    #[inline(always)]
    pub fn is_low(&self) -> bool {
        self._is_low()
    }
}

impl<const P: char, const N: u8, MODE> Pin<P, N, Input<MODE>> {
    #[inline(always)]
    pub fn is_high(&self) -> bool {
        !self.is_low()
    }

    #[inline(always)]
    pub fn is_low(&self) -> bool {
        self._is_low()
    }
}

macro_rules! gpio {
    ($GPIOX:ident, $gpiox:ident, $PEPin:ident, $port_id:expr, $PXn:ident, [
        $($PXi:ident: ($pxi:ident, $i:expr $(, $MODE:ty)?),)+
    ]) => {
        /// GPIO
        pub mod $gpiox {
            use crate::pac::$GPIOX;
            use crate::rcc::{Enable, Reset};
            use super::{
                Floating, Input,
            };

            /// GPIO parts
            pub struct Parts {
                $(
                    /// Pin
                    pub $pxi: $PXi $(<$MODE>)?,
                )+
            }

            impl super::GpioExt for $GPIOX {
                type Parts = Parts;

                fn split(self) -> Parts {
                    unsafe {
                        <$GPIOX>::enable_unchecked();
                        <$GPIOX>::reset_unchecked();
                    }

                    Parts {
                        $(
                            $pxi: $PXi::new(),
                        )+
                    }
                }
            }

            pub type $PXn<MODE> = super::PEPin<$port_id, MODE>;

            $(
                pub type $PXi<MODE = Input<Floating>> = super::Pin<$port_id, $i, MODE>;
            )+

        }

        pub use $gpiox::{ $($PXi,)+ };
    }
}

gpio!(GPIOA, gpioa, PA, 'A', PAn, [
    PA0: (pa0, 0),
    PA1: (pa1, 1),
    PA2: (pa2, 2),
    PA3: (pa3, 3),
    PA4: (pa4, 4),
    PA5: (pa5, 5),
    PA6: (pa6, 6),
    PA7: (pa7, 7),
    PA8: (pa8, 8),
    PA9: (pa9, 9),
    PA10: (pa10, 10),
    PA11: (pa11, 11),
    PA12: (pa12, 12),
    PA13: (pa13, 13, super::Debugger), // SWDIO, PullUp VeryHigh speed
    PA14: (pa14, 14, super::Debugger), // SWCLK, PullDown
    PA15: (pa15, 15),
]);

gpio!(GPIOB, gpiob, PB, 'B', PBn, [
    PB0: (pb0, 0),
    PB1: (pb1, 1),
    PB2: (pb2, 2),
    PB3: (pb3, 3),
    PB4: (pb4, 4),
    PB5: (pb5, 5),
    PB6: (pb6, 6),
    PB7: (pb7, 7),
    PB8: (pb8, 8),
    PB9: (pb9, 9),
    PB10: (pb10, 10),
    PB11: (pb11, 11),
    PB12: (pb12, 12),
    PB13: (pb13, 13),
    PB14: (pb14, 14),
    PB15: (pb15, 15),
]);

// TODO: GPIOC, GPIOD

struct Gpio<const P: char>;
impl<const P: char> Gpio<P> {
    const fn ptr() -> *const crate::pac::gpioa::RegisterBlock {
        match P {
            'A' => crate::pac::GPIOA::ptr(),
            'B' => crate::pac::GPIOB::ptr() as _,
            'C' => crate::pac::GPIOC::ptr() as _,
            'D' => crate::pac::GPIOD::ptr() as _,
            _ => crate::pac::GPIOA::ptr(),
        }
    }
}
