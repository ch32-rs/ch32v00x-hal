//! GPIO and Alternate function

use core::fmt;
use core::marker::PhantomData;

pub use embedded_hal_02::digital::v2::PinState;

mod convert;
mod hal_02;
mod hal_1;
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
    fn split(self, rcc: &mut crate::rcc::Rcc) -> Self::Parts;
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
/// - `P` is port name: `A` for GPIOA, `C` for GPIOC, etc.
/// - `N` is pin number: from `0` to `7`.
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
        unsafe {
            (*Gpio::<P>::ptr()).cfglr.modify(|r, w| {
                w.bits((r.bits() & !(0b11 << Self::OFFSET)) | ((speed as u32) << Self::OFFSET))
            })
        };

        self
    }
}

// NOTE: No internal_pull_up and internal_pull_down for Output<OpenDrain>

impl<const P: char, const N: u8> Pin<P, N, Alternate<PushPull>> {
    /// Set pin speed
    pub fn set_speed(self, speed: Speed) -> Self {
        unsafe {
            (*Gpio::<P>::ptr()).cfglr.modify(|r, w| {
                w.bits((r.bits() & !(0b11 << Self::OFFSET)) | ((speed as u32) << Self::OFFSET))
            })
        };

        self
    }
}

impl<const P: char, const N: u8> Pin<P, N, Alternate<PushPull>> {
    /// Turns pin alternate configuration pin into open drain
    pub fn set_open_drain(self) -> Pin<P, N, Alternate<OpenDrain>> {
        // CNFy
        let offset = Self::OFFSET + 2;

        unsafe {
            (*Gpio::<P>::ptr())
                .cfglr
                .modify(|r, w| w.bits((r.bits() & !(0b11 << offset)) | (0b11 << offset)))
        };

        Pin::new()
    }
}

// TODO: Erase pin number, Erase pin number and port number

impl<const P: char, const N: u8, MODE> Pin<P, N, MODE> {
    /// Offset into the config register
    const OFFSET: u8 = N * 4;

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
        self.set_state(!self.get_state())
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
    ($GPIOX:ident, $gpiox:ident, $PEPin:ident, $port_id:expr, $PXn:ident, $enable:ident, $reset:ident, [
        $($PXi:ident: ($pxi:ident, $i:expr $(, $MODE:ty)?),)+
    ]) => {
        /// GPIO
        pub mod $gpiox {
            use crate::pac::$GPIOX;
            use crate::rcc::{Rcc, Enable, Reset};
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

                fn split(self, rcc: &mut Rcc) -> Parts {
                    // Power on peripheral and reset it
                    $GPIOX :: enable(&mut rcc.apb2);
                    $GPIOX :: reset(&mut rcc.apb2);

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

// Only 2 pins are exposed on any of the package variants
gpio!(GPIOA, gpioa, PA, 'A', PAn, iopaen, ioparst, [
    // PA0: (pa0, 0),
    PA1: (pa1, 1),
    PA2: (pa2, 2),
    // PA3: (pa3, 3),
    // PA4: (pa4, 4),
    // PA5: (pa5, 5),
    // PA6: (pa6, 6),
    // PA7: (pa7, 7),
]);

gpio!(GPIOC, gpioc, PC, 'C', PCn, iopcen, iopcrst, [
    PC0: (pc0, 0),
    PC1: (pc1, 1),
    PC2: (pc2, 2),
    PC3: (pc3, 3),
    PC4: (pc4, 4),
    PC5: (pc5, 5),
    PC6: (pc6, 6),
    PC7: (pc7, 7),
]);

gpio!(GPIOD, gpiod, PD, 'D', PDn, iopden, iopdrst, [
    PD0: (pd0, 0),
    PD1: (pd1, 1, super::Alternate), // By default in SWD mode
    PD2: (pd2, 2),
    PD3: (pd3, 3),
    PD4: (pd4, 4),
    PD5: (pd5, 5),
    PD6: (pd6, 6),
    PD7: (pd7, 7),
]);

struct Gpio<const P: char>;
impl<const P: char> Gpio<P> {
    const fn ptr() -> *const crate::pac::gpioa::RegisterBlock {
        match P {
            'A' => crate::pac::GPIOA::ptr(),
            'C' => crate::pac::GPIOC::ptr() as _,
            'D' => crate::pac::GPIOD::ptr() as _,
            _ => crate::pac::GPIOA::ptr(),
        }
    }
}
use crate::serial;
impl serial::Ck<0> for gpiod::PD4<Alternate<PushPull>> {}
impl serial::Tx<0> for gpiod::PD5<Alternate<PushPull>> {}
impl serial::Rx<0> for gpiod::PD6<Input<Floating>> {}
impl serial::Cts<0> for gpiod::PD3<Input<Floating>> {}
impl serial::Rts<0> for gpioc::PC2<Alternate<PushPull>> {}

impl serial::Ck<1> for gpiod::PD7<Alternate<PushPull>> {}
impl serial::Tx<1> for gpiod::PD0<Alternate<PushPull>> {}
impl serial::Rx<1> for gpiod::PD1<Input<Floating>> {}
impl serial::Cts<1> for gpioc::PC3<Input<Floating>> {}
impl serial::Rts<1> for gpioc::PC2<Alternate<PushPull>> {}

impl serial::Ck<2> for gpiod::PD7<Alternate<PushPull>> {}
impl serial::Tx<2> for gpiod::PD6<Alternate<PushPull>> {}
impl serial::Rx<2> for gpiod::PD5<Input<Floating>> {}
impl serial::Cts<2> for gpioc::PC6<Input<Floating>> {}
impl serial::Rts<2> for gpioc::PC7<Alternate<PushPull>> {}

impl serial::Ck<3> for gpioc::PC5<Alternate<PushPull>> {}
impl serial::Tx<3> for gpioc::PC0<Alternate<PushPull>> {}
impl serial::Rx<3> for gpioc::PC1<Input<Floating>> {}
impl serial::Cts<3> for gpioc::PC6<Input<Floating>> {}
impl serial::Rts<3> for gpioc::PC7<Alternate<PushPull>> {}
