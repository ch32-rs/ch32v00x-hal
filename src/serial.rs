//! Universal Synchronous Asynchronous Receiver Transmitter (USART)

use core::fmt;
use core::marker::PhantomData;
use core::ops::Deref;
use core::ops::DerefMut;
use core::pin::Pin;
use core::ptr;

use crate::gpio::Floating;
use crate::gpio::Input;
use crate::hal::prelude::*;
use crate::hal::serial;
use crate::pac;
use crate::rcc::{BusClock, Enable, Reset};
use crate::state;

use crate::pac::{AFIO, RCC, UART4, UART5, UART7, USART1, USART2, USART3};

use crate::gpio::{self, Alternate};

use crate::rcc::Clocks;
use crate::{BitsPerSecond, U32Ext};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DataBits {
    DataBits8,
    DataBits9,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Parity {
    ParityNone,
    ParityEven,
    ParityOdd,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StopBits {
    #[doc = "1 stop bit"]
    STOP1,
    #[doc = "0.5 stop bits"]
    STOP0P5,
    #[doc = "2 stop bits"]
    STOP2,
    #[doc = "1.5 stop bits"]
    STOP1P5,
}

impl StopBits {
    fn to_raw(self) -> u8 {
        match self {
            StopBits::STOP1 => 0,
            StopBits::STOP0P5 => 1,
            StopBits::STOP2 => 2,
            StopBits::STOP1P5 => 3,
        }
    }
}

#[non_exhaustive]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Config {
    pub baudrate: u32,
    pub data_bits: DataBits,
    pub stop_bits: StopBits,
    pub parity: Parity,
}

impl Default for Config {
    // 115200 8N1
    fn default() -> Self {
        Self {
            baudrate: 115200,
            data_bits: DataBits::DataBits8,
            stop_bits: StopBits::STOP1,
            parity: Parity::ParityNone,
        }
    }
}

/// Serial error
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    /// Framing error
    Framing,
    /// Noise error
    Noise,
    /// RX buffer overrun
    Overrun,
    /// Parity check error
    Parity,
    /// Buffer too large for DMA
    BufferTooLong,
}

pub trait TxPin<USART> {
    // Set up pins for AFIO remap function
    fn setup(&self);
}

impl TxPin<USART1> for gpio::PA9<Alternate> {
    fn setup(&self) {}
}

impl TxPin<USART1> for gpio::PB6<Alternate> {
    fn setup(&self) {
        if AFIO::is_disabled() {
            unsafe {
                AFIO::enable_unchecked();
                USART1::reset_unchecked();
            }
        }
        let afio = unsafe { &*AFIO::ptr() };
        afio.pcfr.modify(|_, w| w.usart1rm().set_bit());
    }
}

pub struct UartTx<USART, P: TxPin<USART>> {
    phantom: PhantomData<(USART, P)>,
}

impl<USART, PIN> UartTx<USART, PIN>
where
    PIN: TxPin<USART>,
    USART: Instance,
{
    pub fn new(usart: USART, pin: PIN, clocks: &Clocks, config: Config) -> Self {
        unsafe {
            USART::enable_unchecked();
            USART::reset_unchecked();
        }

        // setup remap
        pin.setup();

        let usart = unsafe { &*USART::ptr() };

        // baudrate calculation
        let apbclk = USART::clock(&clocks).raw(); // pclk2 or pclk1
        let integer_divider = (25 * apbclk) / (4 * config.baudrate);
        let mut tmpreg = (integer_divider / 100) << 4;
        let fractional_divider = integer_divider - (100 * (tmpreg >> 4));
        tmpreg |= (((fractional_divider * 16) + 50) / 100) & 0x0F;

        usart.brr.write(|w| unsafe { w.bits(tmpreg) });

        // stopbits
        usart
            .ctlr2
            .modify(|_, w| unsafe { w.stop().bits(config.stop_bits.to_raw()) });

        // databits, parity
        usart.ctlr1.modify(|_, w| {
            w.m()
                .bit(config.data_bits == DataBits::DataBits9)
                .pce()
                .bit(config.parity != Parity::ParityNone)
                .ps()
                .bit(config.parity == Parity::ParityOdd)
                .te()
                .set_bit()
        });

        // TODO: no dma, no flow control
        usart.ctlr3.modify(|_, w| {
            w.ctse()
                .clear_bit()
                .rtse()
                .clear_bit()
                .dmat()
                .clear_bit()
                .dmar()
                .clear_bit()
        });

        // enable
        usart.ctlr1.modify(|_, w| w.ue().set_bit());

        Self {
            phantom: PhantomData,
        }
    }
}

impl<USART, PIN> embedded_hal::serial::Write<u8> for UartTx<USART, PIN>
where
    PIN: TxPin<USART>,
    USART: Instance,
{
    type Error = Error;

    fn write(&mut self, word: u8) -> nb::Result<(), Self::Error> {
        let usart = unsafe { &*USART::ptr() };
        if usart.statr.read().tc().bit_is_clear() {
            Err(nb::Error::WouldBlock)
        } else {
            usart.datar.write(|w| unsafe { w.dr().bits(word as _) });
            Ok(())
        }
    }

    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        let usart = unsafe { &*USART::ptr() };
        if usart.statr.read().tc().bit_is_set() {
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }
}

impl<USART, PIN> core::fmt::Write for UartTx<USART, PIN>
where
    PIN: TxPin<USART> + 'static,
    USART: Instance + 'static,
{
    fn write_str(&mut self, s: &str) -> fmt::Result {
        (self as &mut dyn embedded_hal::serial::Write<u8, Error = _>).write_str(s)
    }
}

/// embedded-hal alpha serial implementation
impl embedded_hal_alpha::serial::Error for Error {
    fn kind(&self) -> embedded_hal_alpha::serial::ErrorKind {
        match self {
            Error::Framing => embedded_hal_alpha::serial::ErrorKind::FrameFormat,
            Error::Noise => embedded_hal_alpha::serial::ErrorKind::Noise,
            Error::Overrun => embedded_hal_alpha::serial::ErrorKind::Overrun,
            Error::Parity => embedded_hal_alpha::serial::ErrorKind::Parity,
            Error::BufferTooLong => embedded_hal_alpha::serial::ErrorKind::Other,
        }
    }
}

impl<USART, PIN> embedded_hal_alpha::serial::ErrorType for UartTx<USART, PIN>
where
    PIN: TxPin<USART>,
{
    type Error = Error;
}

impl<USART, PIN> embedded_hal_alpha::serial::Write<u8> for UartTx<USART, PIN>
where
    PIN: TxPin<USART>,
    USART: Instance,
{
    fn write(&mut self, buffer: &[u8]) -> Result<(), Self::Error> {
        todo!()
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        todo!()
    }
}

/// Implemented by all USART instances
pub trait Instance: Deref<Target = pac::usart1::RegisterBlock> + Enable + Reset + BusClock {
    fn ptr() -> *const pac::usart1::RegisterBlock;
}

macro_rules! impl_instance {
    ($(
        $USARTX:ident: ($usartXsel:ident),
    )+) => {
        $(
            impl Instance for $USARTX {
                fn ptr() -> *const pac::usart1::RegisterBlock {
                    $USARTX::ptr()
                }
            }
        )+
    }
}

impl_instance! {
    USART1: (usart1sel),
    USART2: (usart2sel),
}

#[cfg(any(feature = "ch32v203c8", feature = "ch32v203rb"))]
impl_instance! {
    USART3: (usart3sel),
    USART4: (usart4sel),
}
