//! Universal Synchronous Asynchronous Receiver Transmitter (USART)

use crate::pac::{AFIO, USART1};
use crate::rcc::{BusClock, Clocks, Enable, Rcc, Reset};
use core::convert::Infallible;
use core::fmt;
use embedded_hal_02::serial::{Read, Write};

pub trait Ck<const REMAP: u8> {
    fn enable(usart: &USART1) {
        usart.ctlr2.modify(|_, w| w.clken().set_bit());
    }
}

pub trait Tx<const REMAP: u8> {
    fn enable(usart: &USART1) {
        usart.ctlr1.modify(|_, w| w.te().set_bit());
    }
}

pub trait Rx<const REMAP: u8> {
    fn enable(usart: &USART1) {
        usart.ctlr1.modify(|_, w| w.re().set_bit());
    }
}

pub trait Cts<const REMAP: u8> {
    fn enable(usart: &USART1) {
        usart.ctlr3.modify(|_, w| w.ctse().set_bit());
    }
}

pub trait Rts<const REMAP: u8> {
    fn enable(usart: &USART1) {
        usart.ctlr3.modify(|_, w| w.rtse().set_bit());
    }
}

pub struct NoCk {}
pub struct NoTx {}
pub struct NoRx {}
pub struct NoCts {}
pub struct NoRts {}

impl<const T: u8> Ck<{ T }> for NoCk {
    fn enable(usart: &USART1) {
        usart.ctlr2.modify(|_, w| w.clken().clear_bit());
    }
}

impl<const T: u8> Tx<{ T }> for NoTx {
    fn enable(usart: &USART1) {
        usart.ctlr1.modify(|_, w| w.te().clear_bit());
    }
}

impl<const T: u8> Rx<{ T }> for NoRx {
    fn enable(usart: &USART1) {
        usart.ctlr1.modify(|_, w| w.re().clear_bit());
    }
}

impl<const T: u8> Cts<{ T }> for NoCts {
    fn enable(usart: &USART1) {
        usart.ctlr3.modify(|_, w| w.ctse().clear_bit());
    }
}

impl<const T: u8> Rts<{ T }> for NoRts {
    fn enable(usart: &USART1) {
        usart.ctlr3.modify(|_, w| w.rtse().clear_bit());
    }
}

/// Serial error
#[derive(Debug)]
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
}

pub trait UsartExt {
    fn usart<const REMAP: u8, TX: Tx<REMAP>, RX: Rx<REMAP>>(
        self,
        tx: TX,
        rx: RX,
        config: Config,
        rcc: &mut Rcc,
        clocks: &Clocks,
    ) -> Usart<NoCk, TX, RX, NoCts, NoRts>;
}

pub struct Usart<CK, TX, RX, CTS, RTS> {
    usart: USART1,
    ck: CK,
    tx: TX,
    rx: RX,
    cts: CTS,
    rts: RTS,
}

impl<CK, TX, RX, CTS, RTS> Usart<CK, TX, RX, CTS, RTS> {
    pub fn free(self) -> (CK, TX, RX, CTS, RTS, USART1) {
        (self.ck, self.tx, self.rx, self.cts, self.rts, self.usart)
    }
}

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

impl UsartExt for USART1 {
    fn usart<const REMAP: u8, TX: Tx<REMAP>, RX: Rx<REMAP>>(
        self,
        tx: TX,
        rx: RX,
        config: Config,
        rcc: &mut Rcc,
        clocks: &Clocks,
    ) -> Usart<NoCk, TX, RX, NoCts, NoRts> {
        let usart = self;

        USART1::enable(&mut rcc.apb2);
        USART1::reset(&mut rcc.apb2);

        AFIO::enable(&mut rcc.apb2);

        let apbclk = USART1::clock(&clocks).raw();
        let integer_divider = (25 * apbclk) / (4 * config.baudrate);
        let div_m = integer_divider / 100;
        let div_f = integer_divider - 100 * div_m;

        usart.brr.write(|w| {
            w.div_fraction()
                .variant(div_f as u8)
                .div_mantissa()
                .variant(div_m as u16)
        });

        let afio = unsafe { &(*AFIO::ptr()) };

        afio.pcfr.modify(|_, w| {
            w.usart1rm()
                .bit(REMAP & 0b1 == 1)
                .usart1remap1()
                .bit((REMAP & 0b10) >> 1 == 1)
        });

        // set stop bits
        usart
            .ctlr2
            .modify(|_, w| w.stop().variant(config.stop_bits.to_raw()));

        usart.ctlr1.modify(|_, w| {
            w.m()
                .bit(config.data_bits == DataBits::DataBits9)
                .pce()
                .bit(config.parity != Parity::ParityNone)
                .ps()
                .bit(config.parity == Parity::ParityOdd)
        });

        TX::enable(&usart);
        RX::enable(&usart);

        // enable usart
        usart.ctlr1.modify(|_, w| w.ue().set_bit());

        Usart {
            ck: NoCk {},
            tx,
            rx,
            cts: NoCts {},
            rts: NoRts {},
            usart,
        }
    }
}

impl<CK, TX, RX, CTS, RTS> Usart<CK, TX, RX, CTS, RTS> {
    pub fn use_clock<const REMAP: u8>(&mut self, clock: CK)
    where
        CK: Ck<REMAP>,
        TX: Tx<REMAP>,
        RX: Rx<REMAP>,
    {
        CK::enable(&self.usart);
        self.ck = clock;
    }

    pub fn write_u16(&mut self, word: u16) -> nb::Result<(), Infallible> {
        if self.usart.statr.read().txe().bit_is_set() {
            self.usart.datar.write(|w| w.dr().variant(word));
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }

    pub fn flush(&mut self) -> nb::Result<(), Infallible> {
        if self.usart.statr.read().tc().bit_is_set() {
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }

    pub fn read_u16(&mut self) -> nb::Result<u16, Error> {
        let statr = self.usart.statr.read();

        // Check for any errors
        let err = if statr.pe().bit_is_set() {
            Some(Error::Parity)
        } else if statr.fe().bit_is_set() {
            Some(Error::Framing)
        } else if statr.ne().bit_is_set() {
            Some(Error::Noise)
        } else if statr.ore().bit_is_set() {
            Some(Error::Overrun)
        } else {
            None
        };

        if let Some(err) = err {
            // Some error occurred. In order to clear that error flag, you have to
            // do a read from the statr register followed by a read from the datar register.
            let _ = self.usart.statr.read();
            let _ = self.usart.datar.read();
            Err(nb::Error::Other(err))
        } else {
            // Check if a byte is available
            if statr.rxne().bit_is_set() {
                // Read the received byte
                Ok(self.usart.datar.read().dr().bits())
            } else {
                Err(nb::Error::WouldBlock)
            }
        }
    }
}

impl<CK, TX, RX, CTS, RTS> core::fmt::Write for Usart<CK, TX, RX, CTS, RTS>
where
    CK: 'static,
    TX: 'static,
    RX: 'static,
    CTS: 'static,
    RTS: 'static,
{
    fn write_str(&mut self, s: &str) -> fmt::Result {
        (self as &mut dyn embedded_hal_02::serial::Write<u8, Error = _>).write_str(s)
    }
}

impl<CK, TX, RX, CTS, RTS> Write<u8> for Usart<CK, TX, RX, CTS, RTS> {
    type Error = Infallible;

    fn write(&mut self, word: u8) -> nb::Result<(), Self::Error> {
        self.write_u16(word as u16)
    }

    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        self.flush()
    }
}

impl<CK, TX, RX, CTS, RTS> Write<u16> for Usart<CK, TX, RX, CTS, RTS> {
    type Error = Infallible;

    fn write(&mut self, word: u16) -> nb::Result<(), Self::Error> {
        self.write_u16(word)
    }

    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        self.flush()
    }
}

impl<CK, TX, RX, CTS, RTS> Read<u8> for Usart<CK, TX, RX, CTS, RTS> {
    type Error = Error;

    fn read(&mut self) -> nb::Result<u8, Self::Error> {
        self.read_u16().map(|word16| word16 as u8)
    }
}

impl<CK, TX, RX, CTS, RTS> Read<u16> for Usart<CK, TX, RX, CTS, RTS> {
    type Error = Error;

    fn read(&mut self) -> nb::Result<u16, Self::Error> {
        self.read_u16()
    }
}
