//! Reset and clock control.

use core::ops::Div;

mod enable;

use ch32v0::{ch32v003::rcc::cfgr0::CFGR0_SPEC, Readable, Reg, Writable};
use fugit::{HertzU32 as Hertz, RateExtU32};

use crate::pac::{rcc, RCC};

/// Typical output frequency of the HSI oscillator.
const HSI_FREQUENCY: Hertz = Hertz::from_raw(24_000_000);

/// Extension trait that constrains the `RCC` peripheral
pub trait RccExt {
    /// Constrains the `RCC` peripheral so it plays nicely with the other abstractions
    fn constrain(self) -> Rcc;
}

impl RccExt for RCC {
    fn constrain(self) -> Rcc {
        Rcc {
            ahb: AHB::new(),
            apb1: APB1::new(),
            apb2: APB2::new(),
            config: Config::default(),
        }
    }
}

/// Constrained RCC peripheral
pub struct Rcc {
    /// Advanced High-Performance Bus (AHB) registers
    pub ahb: AHB,
    /// Advanced Peripheral Bus 1 (APB1) registers
    pub apb1: APB1,
    /// Advanced Peripheral Bus 2 (APB2) registers
    pub apb2: APB2,
    pub config: Config,
}

macro_rules! bus_struct {
    ($($busX:ident => ($EN:ident, $en:ident, $($RST:ident, $rst:ident,)? $doc:literal),)+) => {
        $(
            #[doc = $doc]
            pub struct $busX {
                _0: (),
            }

            impl $busX {
                pub(crate) fn new() -> Self {
                    Self { _0: () }
                }

                /// Enable register
                pub(crate) fn enr(&self) -> &rcc::$EN {
                    // NOTE(unsafe) this proxy grants exclusive access to this register
                    unsafe { &(*RCC::ptr()).$en }
                }

                $(
                    /// Reset register
                    pub(crate) fn rstr(&self) -> &rcc::$RST {
                        // NOTE(unsafe) this proxy grants exclusive access to this register
                        unsafe { &(*RCC::ptr()).$rst }
                    }
                )?
            }
        )+
    };
}

bus_struct! {
    APB1 => (APB1PCENR, apb1pcenr, APB1PRSTR, apb1prstr, "Advanced Peripheral Bus 1 (APB1) registers"),
    APB2 => (APB2PCENR, apb2pcenr, APB2PRSTR, apb2prstr, "Advanced Peripheral Bus 2 (APB2) registers"),
    AHB => (AHBPCENR, ahbpcenr, "Advanced High-performance Bus (AHB) registers"),
}

// clock config

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HSEConfig {
    pub frequency: Hertz,
    pub source: HSESrc,
}

impl Default for HSEConfig {
    fn default() -> Self {
        Self {
            frequency: Hertz::MHz(24),
            source: HSESrc::Crystal,
        }
    }
}

/// HSE clock source
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HSESrc {
    /// Crystal/ceramic resonator
    Crystal,
    /// External clock source, HSE bypassed
    Bypass,
}

impl Default for HSESrc {
    fn default() -> Self {
        Self::Crystal
    }
}

/// Source of core clock signal
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum ClockSrc {
    /// 24MHz internal RC oscillator
    Hsi = 0b00,
    /// High speed external oscillator, 5-25Mhz
    Hse = 0b01,
    /// Internal phase locked loop
    Pll = 0b10,
}

/// AMBA High-performance bus (AHB) prescaler
#[derive(Clone, Copy, PartialEq)]
pub enum AHBPrescaler {
    NotDivided = 0b0000,
    Div2 = 0b0001,
    Div3 = 0b0010,
    Div4 = 0b0011,
    Div5 = 0b0100,
    Div6 = 0b0101,
    Div7 = 0b0110,
    Div8 = 0b0111,
    // Div2 = 0b1000 Two ways to divide by 2, 4, 8
    Div16 = 0b1011,
    Div32 = 0b1100,
    Div64 = 0b1101,
    Div128 = 0b1110,
    Div256 = 0b1111,
}

impl Div<AHBPrescaler> for Hertz {
    type Output = Hertz;

    fn div(self, rhs: AHBPrescaler) -> Self::Output {
        match rhs {
            AHBPrescaler::NotDivided => self,
            AHBPrescaler::Div2 => self / 2,
            AHBPrescaler::Div3 => self / 3,
            AHBPrescaler::Div4 => self / 4,
            AHBPrescaler::Div5 => self / 5,
            AHBPrescaler::Div6 => self / 6,
            AHBPrescaler::Div7 => self / 7,
            AHBPrescaler::Div8 => self / 8,
            AHBPrescaler::Div16 => self / 16,
            AHBPrescaler::Div32 => self / 32,
            AHBPrescaler::Div64 => self / 64,
            AHBPrescaler::Div128 => self / 128,
            AHBPrescaler::Div256 => self / 256,
        }
    }
}

/// Source for the internal phase locked loop
#[derive(Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum PLLSrc {
    /// PLL is fed from the external high speed clock
    Hse = 0b1,
    /// PLL is fed from the interla high speed clock
    Hsi = 0b0,
}

/// Microcontroller clock output
///
/// Value on reset: None
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum MCO {
    /// No clock
    None = 0b000,
    /// System clock selected
    Sysclk = 0b100,
    /// HSI clock selected
    Hsi = 0b101,
    /// HSE oscillator clock selected
    Hse = 0b110,
    /// PLL clock selected
    Pll = 0b111,
}

/// Clock configuration
#[derive(Clone, Copy)]
pub struct Config {
    /// High speed external clock
    pub hse: Option<HSEConfig>,
    /// Phase locked loop (2x multiplier)
    pub pll: PLLSrc,
    /// Enable internal 128Khz clock. Cannot be used as core clock source
    pub enable_lsi: bool,
    /// Which clock feeds the core frequency
    pub mux: ClockSrc,
    /// AHB bus frequency prescaler
    pub ahb_pre: AHBPrescaler,
    /// Clock output configuration
    pub mco: MCO,
}

impl Default for Config {
    // 24MHz HSI
    fn default() -> Self {
        Self {
            hse: None,
            pll: PLLSrc::Hsi,
            enable_lsi: false,
            mux: ClockSrc::Hsi,
            ahb_pre: AHBPrescaler::NotDivided,
            mco: MCO::None,
        }
    }
}

impl Config {
    /// Configure the "mandatory" clocks (`sysclk`, `hclk`, `pclk1` and `pclk2')
    /// and return them via the `Clocks` struct.
    ///
    /// The user shouldn't call freeze more than once as the clocks parameters
    /// cannot be changed after the clocks have started.
    ///
    /// The implementation makes the following choice: HSI is always chosen over
    /// HSE except when HSE is provided. When HSE is provided, HSE is used
    /// wherever it is possible.
    pub fn freeze(self) -> Clocks {
        let rcc = unsafe { &(*RCC::ptr()) };

        let mut clocks = Clocks::default();

        // Helper function to write to a register and block until condition is met
        fn block<REG>(
            reg: &Reg<REG>,
            set: impl Fn(&mut REG::Writer) -> &mut REG::Writer,
            get: impl Fn(REG::Reader) -> bool,
        ) where
            REG: Readable + Writable,
        {
            reg.modify(|_, w| set(w));
            while !get(reg.read()) {}
        }

        // Helper to set clock source blockingly
        fn block_clock(cfgr0: &Reg<CFGR0_SPEC>, src: ClockSrc) {
            block(
                cfgr0,
                |w| w.sw().variant(src as u8),
                |r| r.sws().bits() == src as u8,
            )
        }

        // Ensure HSI is on and switch to it
        block(
            &rcc.ctlr,
            |w| w.hsion().set_bit(),
            |r| r.hsirdy().bit_is_set(),
        );
        block_clock(&rcc.cfgr0, ClockSrc::Hsi);

        // Configure HSE if provided
        if let Some(hse) = self.hse {
            match hse.source {
                HSESrc::Crystal => rcc.ctlr.modify(|_, w| w.hsebyp().clear_bit()),
                HSESrc::Bypass => rcc.ctlr.modify(|_, w| w.hsebyp().set_bit()),
            }
            // Start HSE, wait for it to stabilize
            block(
                &rcc.ctlr,
                |w| w.hseon().set_bit(),
                |r| r.hserdy().bit_is_set(),
            );
            clocks.hse = Some(hse.frequency);
        }

        // Configure HCLK
        // TODO: ADCPRE
        rcc.cfgr0
            .modify(|_, w| w.hpre().variant(self.ahb_pre as u8));

        // Enable PWR domain
        rcc.apb1pcenr.modify(|_, w| w.pwren().set_bit());
        // Enable editing backup_domain RCC.BDCTLR
        // pwr.ctlr.modify(|_, w| w.dbp().set_bit());

        match (self.mux, self.pll) {
            (ClockSrc::Hse, _) => {
                block_clock(&rcc.cfgr0, ClockSrc::Hse);
                clocks.sysclk = clocks.hse.unwrap();
            }
            (ClockSrc::Hsi, _) => {
                block_clock(&rcc.cfgr0, ClockSrc::Hsi);
                clocks.sysclk = HSI_FREQUENCY;
            }
            (ClockSrc::Pll, src) => {
                // Disable PLL, PLLMUL, PLLXTPRE, PLLSRC can only be written when PLL is off
                rcc.ctlr.modify(|_, w| w.pllon().clear_bit());

                match src {
                    PLLSrc::Hsi => {
                        // HSI is used as PLL source
                        rcc.cfgr0.modify(|_, w| w.pllsrc().clear_bit());
                        clocks.sysclk = HSI_FREQUENCY * 2;
                    }
                    PLLSrc::Hse => {
                        // HSE is used as PLL source
                        rcc.cfgr0.modify(|_, w| w.pllsrc().set_bit());
                        clocks.sysclk = clocks.hse.unwrap() * 2;
                    }
                }
                clocks.pllclk = Some(clocks.sysclk);

                // Enable PLL
                block(
                    &rcc.ctlr,
                    |w| w.pllon().set_bit(),
                    |r| r.pllrdy().bit_is_set(),
                );
                block_clock(&rcc.cfgr0, ClockSrc::Pll);
            }
        }

        // Calculate AHB and APB speeds
        clocks.hclk = clocks.sysclk / self.ahb_pre;

        // Configure low speed internal RC (128khz)
        if self.enable_lsi {
            block(
                &rcc.rstsckr,
                |w| w.lsion().set_bit(),
                |r| r.lsirdy().bit_is_set(),
            );
        }

        // Enable clock output
        rcc.cfgr0.modify(|_, w| w.mco().variant(self.mco as u8));

        // Whats up with this? From 20x hal
        unsafe {
            qingke::riscv::asm::delay(16);
        }

        clocks
    }
}

/// Frozen clock frequencies
///
/// The existence of this value indicates that the clock configuration can no longer be changed
#[derive(Clone, Copy, Debug)]
pub struct Clocks {
    pub sysclk: Hertz,
    pub hclk: Hertz,
    pub pllclk: Option<Hertz>,
    pub hse: Option<Hertz>,
    pub lsi: Option<Hertz>,
}

impl Clocks {
    /// Returns the frequency of the AHB1
    pub fn hclk(&self) -> Hertz {
        self.hclk
    }

    /// Returns the system (core) frequency
    pub fn sysclk(&self) -> Hertz {
        self.sysclk
    }

    /// Returns the frequency of the `HSE` if `Some`, else `None`.
    pub fn hse(&self) -> Option<Hertz> {
        self.hse
    }

    /// Returns the frequency of the `LSI` if `Some`, else `None`.
    pub fn lsi(&self) -> Option<Hertz> {
        self.lsi
    }
}

impl Default for Clocks {
    fn default() -> Self {
        Clocks {
            sysclk: 24.MHz(),
            hclk: 8.MHz(),
            pllclk: None,
            hse: None,
            lsi: None,
        }
    }
}

/// Frequency on bus that peripheral is connected in
pub trait BusClock {
    /// Calculates frequency depending on `Clock` state
    fn clock(clocks: &Clocks) -> Hertz;
}

/// Frequency on bus that timer is connected in
pub trait BusTimerClock {
    /// Calculates base frequency of timer depending on `Clock` state
    fn timer_clock(clocks: &Clocks) -> Hertz;
}

impl<T> BusClock for T
where
    T: RccBus,
    T::Bus: BusClock,
{
    fn clock(clocks: &Clocks) -> Hertz {
        T::Bus::clock(clocks)
    }
}

impl<T> BusTimerClock for T
where
    T: RccBus,
    T::Bus: BusTimerClock,
{
    fn timer_clock(clocks: &Clocks) -> Hertz {
        T::Bus::timer_clock(clocks)
    }
}

impl BusClock for AHB {
    fn clock(clocks: &Clocks) -> Hertz {
        clocks.hclk
    }
}

impl BusClock for APB1 {
    fn clock(clocks: &Clocks) -> Hertz {
        clocks.hclk
    }
}

impl BusClock for APB2 {
    fn clock(clocks: &Clocks) -> Hertz {
        clocks.hclk
    }
}

/// Bus associated to peripheral
pub trait RccBus: crate::Sealed {
    /// Bus type;
    type Bus;
}

/// Enable/disable peripheral
pub trait Enable: RccBus {
    /// Enables peripheral
    fn enable(bus: &mut Self::Bus);

    /// Disables peripheral
    fn disable(bus: &mut Self::Bus);

    /// Check if peripheral enabled
    fn is_enabled() -> bool;

    /// Check if peripheral disabled
    fn is_disabled() -> bool;

    /// # Safety
    ///
    /// Enables peripheral. Takes access to RCC internally
    unsafe fn enable_unchecked();

    /// # Safety
    ///
    /// Disables peripheral. Takes access to RCC internally
    unsafe fn disable_unchecked();
}

/// Reset peripheral
pub trait Reset: RccBus {
    /// Resets peripheral
    fn reset(bus: &mut Self::Bus);

    /// # Safety
    ///
    /// Resets peripheral. Takes access to RCC internally
    unsafe fn reset_unchecked();
}
