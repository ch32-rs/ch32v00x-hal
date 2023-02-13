//! Reset and clock control.

use core::cmp::min;
use core::mem;
use core::ops::{Div, Mul};

mod enable;

use fugit::{HertzU32 as Hertz, RateExtU32};

use crate::pac::{rcc, EXTEND, FLASH, PWR, RCC};

/// Typical output frequency of the HSI oscillator.
const HSI_FREQUENCY: Hertz = Hertz::from_raw(8_000_000);

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
            bdcr: BDCR::new(),
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
    /// RCC Backup Domain
    pub bdcr: BDCR,
    pub config: Config,
}

macro_rules! bus_struct {
    ($($busX:ident => ($EN:ident, $en:ident, $RST:ident, $rst:ident, $doc:literal),)+) => {
        $(
            #[doc = $doc]
            pub struct $busX {
                _0: (),
            }

            impl $busX {
                pub(crate) fn new() -> Self {
                    Self { _0: () }
                }

                pub(crate) fn enr(&self) -> &rcc::$EN {
                    // NOTE(unsafe) this proxy grants exclusive access to this register
                    unsafe { &(*RCC::ptr()).$en }
                }

                pub(crate) fn rstr(&self) -> &rcc::$RST {
                    // NOTE(unsafe) this proxy grants exclusive access to this register
                    unsafe { &(*RCC::ptr()).$rst }
                }
            }
        )+
    };
}

bus_struct! {
    APB1 => (APB1PCENR, apb1pcenr, APB1PRSTR, apb1prstr, "Advanced Peripheral Bus 1 (APB1) registers"),
    APB2 => (APB2PCENR, apb2pcenr, APB2PRSTR, apb2prstr, "Advanced Peripheral Bus 2 (APB2) registers"),
    AHB => (AHBPCENR, ahbpcenr, AHBRSTR, ahbrstr, "Advanced High-performance Bus (AHB) registers"),
}

/// Backup Domain Control register (RCC_BDCR)
pub struct BDCR {
    _0: (),
}

impl BDCR {
    pub(crate) fn new() -> Self {
        Self { _0: () }
    }
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
            frequency: Hertz::MHz(32),
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ClockSrc {
    /// For some variants, HSE must be 32MHz
    HSE,
    /// 8MHz internal RC oscillator
    HSI,
    PLL,
}

/// AHB prescaler
#[derive(Clone, Copy, PartialEq)]
pub enum AHBPrescaler {
    NotDivided = 0b0000,
    Div2 = 0b1000,
    Div4 = 0b1001,
    Div8 = 0b1010,
    Div16 = 0b1011,
    Div64 = 0b1100,
    Div128 = 0b1101,
    Div256 = 0b1110,
    Div512 = 0b1111,
}

impl Div<AHBPrescaler> for Hertz {
    type Output = Hertz;

    fn div(self, rhs: AHBPrescaler) -> Self::Output {
        match rhs {
            AHBPrescaler::NotDivided => self,
            AHBPrescaler::Div2 => self / 2,
            AHBPrescaler::Div4 => self / 4,
            AHBPrescaler::Div8 => self / 8,
            AHBPrescaler::Div16 => self / 16,
            AHBPrescaler::Div64 => self / 64,
            AHBPrescaler::Div128 => self / 128,
            AHBPrescaler::Div256 => self / 256,
            AHBPrescaler::Div512 => self / 512,
        }
    }
}

/// APB prescaler
#[derive(Clone, Copy)]
pub enum APBPrescaler {
    NotDivided = 0b000,
    Div2 = 0b100,
    Div4 = 0b101,
    Div8 = 0b110,
    Div16 = 0b111,
}

impl Div<APBPrescaler> for Hertz {
    type Output = Hertz;

    fn div(self, rhs: APBPrescaler) -> Self::Output {
        match rhs {
            APBPrescaler::NotDivided => self,
            APBPrescaler::Div2 => self / 2,
            APBPrescaler::Div4 => self / 4,
            APBPrescaler::Div8 => self / 8,
            APBPrescaler::Div16 => self / 16,
        }
    }
}

pub struct PLLConfig {
    pub src: PLLSrc,
    pub mul: PLLMul,
}

pub enum PLLSrc {
    HSE(HSEPrescaler),
    HSI,
}

#[derive(Clone, Copy)]
pub enum HSEPrescaler {
    NotDivided,
    Div2,
    // #[cfg(any(feature = "d8w", feature = "d8"))]
    Div4,
    Div8,
}

impl HSEPrescaler {
    fn to_taw(self) -> bool {
        match self {
            HSEPrescaler::NotDivided => false,
            HSEPrescaler::Div2 => true,
            HSEPrescaler::Div4 => false,
            HSEPrescaler::Div8 => true,
        }
    }
}

impl Div<HSEPrescaler> for Hertz {
    type Output = Hertz;

    fn div(self, rhs: HSEPrescaler) -> Self::Output {
        match rhs {
            HSEPrescaler::NotDivided => self,
            HSEPrescaler::Div2 => self / 2,
            HSEPrescaler::Div4 => self / 4,
            HSEPrescaler::Div8 => self / 8,
        }
    }
}

/// PLLMUL multiplication factors.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PLLMul {
    #[cfg(not(feature = "d8c"))]
    Mul2 = 0b0000,
    Mul3 = 0b0001,
    Mul4 = 0b0010,
    Mul5 = 0b0011,
    Mul6 = 0b0100,
    #[cfg(feature = "d8c")]
    Mul6_5 = 0b1101,
    Mul7 = 0b0101,
    Mul8 = 0b0110,
    Mul9 = 0b0111,
    Mul10 = 0b1000,
    Mul11 = 0b1001,
    Mul12 = 0b1010,
    Mul13 = 0b1011,
    Mul14 = 0b1100,
    #[cfg(not(feature = "d8c"))]
    Mul15 = 0b1101,
    #[cfg(not(feature = "d8c"))]
    Mul16 = 0b1110,
    #[cfg(not(feature = "d8c"))]
    Mul18 = 0b1111,
    #[cfg(feature = "d8c")]
    Mul15 = 0b1110,
    #[cfg(feature = "d8c")]
    Mul16 = 0b1111,
    #[cfg(feature = "d8c")]
    Mul18 = 0b0000,
}

impl Mul<PLLMul> for Hertz {
    type Output = Hertz;

    fn mul(self, rhs: PLLMul) -> Self::Output {
        match rhs {
            #[cfg(not(feature = "d8c"))]
            PLLMul::Mul2 => self * 2,
            PLLMul::Mul3 => self * 3,
            PLLMul::Mul4 => self * 4,
            PLLMul::Mul5 => self * 5,
            PLLMul::Mul6 => self * 6,
            PLLMul::Mul7 => self * 7,
            PLLMul::Mul8 => self * 8,
            PLLMul::Mul9 => self * 9,
            PLLMul::Mul10 => self * 10,
            PLLMul::Mul11 => self * 11,
            PLLMul::Mul12 => self * 12,
            PLLMul::Mul13 => self * 13,
            PLLMul::Mul14 => self * 14,
            PLLMul::Mul15 => self * 15,
            PLLMul::Mul16 => self * 16,
            PLLMul::Mul18 => self * 18,
            #[cfg(feature = "d8c")]
            PLLMul::Mul6_5 => pll / 2 * 13,
        }
    }
}

/// Clock configuration
pub struct Config {
    pub hse: Option<HSEConfig>,
    /// PLLCLK
    pub pll: PLLConfig,
    pub enable_lsi: bool,
    pub mux: ClockSrc,
    pub ahb_pre: AHBPrescaler,
    pub apb1_pre: APBPrescaler,
    pub apb2_pre: APBPrescaler,
}

impl Config {
    // close presets
    pub const fn sysclk_144m_via_hsi(self) -> Self {
        Self {
            hse: None,
            pll: PLLConfig {
                src: PLLSrc::HSI,
                mul: PLLMul::Mul18,
            },
            enable_lsi: false,
            mux: ClockSrc::PLL,
            ahb_pre: AHBPrescaler::NotDivided,
            apb1_pre: APBPrescaler::NotDivided,
            apb2_pre: APBPrescaler::NotDivided,
        }
    }

    pub const fn sysclk_96m_via_hsi(self) -> Self {
        Self {
            hse: None,
            pll: PLLConfig {
                src: PLLSrc::HSI,
                mul: PLLMul::Mul12,
            },
            enable_lsi: false,
            mux: ClockSrc::PLL,
            ahb_pre: AHBPrescaler::NotDivided,
            apb1_pre: APBPrescaler::NotDivided,
            apb2_pre: APBPrescaler::NotDivided,
        }
    }

    /// 72MHz, USB is not available
    pub const fn sysclk_72m_via_hsi(self) -> Self {
        Self {
            hse: None,
            pll: PLLConfig {
                src: PLLSrc::HSI,
                mul: PLLMul::Mul9,
            },
            enable_lsi: false,
            mux: ClockSrc::PLL,
            ahb_pre: AHBPrescaler::NotDivided,
            apb1_pre: APBPrescaler::NotDivided,
            apb2_pre: APBPrescaler::NotDivided,
        }
    }

    pub const fn sysclk_48m_via_hsi(self) -> Self {
        Self {
            hse: None,
            pll: PLLConfig {
                src: PLLSrc::HSI,
                mul: PLLMul::Mul6,
            },
            enable_lsi: false,
            mux: ClockSrc::PLL,
            ahb_pre: AHBPrescaler::NotDivided,
            apb1_pre: APBPrescaler::NotDivided,
            apb2_pre: APBPrescaler::NotDivided,
        }
    }
}

impl Default for Config {
    // 8MHz HSI
    fn default() -> Self {
        Self {
            hse: None,
            pll: PLLConfig {
                src: PLLSrc::HSI,
                mul: PLLMul::Mul9,
            },
            enable_lsi: false,
            mux: ClockSrc::HSI,
            ahb_pre: AHBPrescaler::NotDivided,
            apb1_pre: APBPrescaler::NotDivided,
            apb2_pre: APBPrescaler::NotDivided,
        }
    }
}

impl Config {
    pub fn freeze(self) -> Clocks {
        let rcc = unsafe { &(*RCC::ptr()) };
        let pwr = unsafe { &(*PWR::ptr()) };
        let flash = unsafe { &(*FLASH::ptr()) };
        // EXTEND is only documented in English version of RM.
        let extend = unsafe { &(*EXTEND::ptr()) };

        let mut clocks = Clocks::default();

        // Turn on HSI, switch to it
        rcc.ctlr.modify(|_, w| w.hsion().set_bit());
        while rcc.ctlr.read().hsirdy().bit_is_clear() {}
        rcc.cfgr0.modify(|_, w| w.sw().hsi());

        // Configure HSE if provided
        if let Some(hse) = self.hse {
            match hse.source {
                HSESrc::Crystal => rcc.ctlr.modify(|_, w| w.hsebyp().clear_bit()),
                HSESrc::Bypass => rcc.ctlr.modify(|_, w| w.hsebyp().set_bit()),
            }
            // Start HSE
            rcc.ctlr.modify(|_, w| w.hseon().set_bit());
            while rcc.ctlr.read().hserdy().bit_is_clear() {}
            clocks.hse = Some(hse.frequency);
        }

        // Configure HCLK, PCLK1, PCLK2
        rcc.cfgr0.modify(|_, w| unsafe {
            w.ppre1()
                .bits(self.apb1_pre as u8)
                .ppre2()
                .bits(self.apb2_pre as u8)
                .hpre()
                .bits(self.ahb_pre as u8)
        });

        // Enable PWR domain
        rcc.apb1pcenr.modify(|_, w| w.pwren().set_bit());
        // Enable editing backup_domain RCC.BDCTLR
        pwr.ctlr.modify(|_, w| w.dbp().set_bit());

        match (self.mux, self.pll) {
            (ClockSrc::HSE, _) => {
                rcc.cfgr0.modify(|_, w| w.sw().hse());
                while !rcc.cfgr0.read().sws().is_hse() {}
                clocks.sysclk = clocks.hse.unwrap();
            }
            (ClockSrc::HSI, _) => {
                rcc.cfgr0.modify(|_, w| w.sw().hsi());
                while !rcc.cfgr0.read().sws().is_hsi() {}
                clocks.sysclk = HSI_FREQUENCY;
            }
            (ClockSrc::PLL, PLLConfig { src, mul }) => {
                // Disable PLL, PLLMUL, PLLXTPRE, PLLSRC can only be written when PLL is off.
                rcc.ctlr.modify(|_, w| w.pllon().clear_bit());

                match src {
                    PLLSrc::HSI => {
                        // HSI is used as PLL source
                        extend.extend_ctr.modify(|_, w| w.pll_hsi_pre().set_bit());
                        rcc.cfgr0.modify(|_, w| unsafe {
                            w.pllmul().bits(mul as u8).pllsrc().bit(false)
                        });
                        clocks.sysclk = HSI_FREQUENCY * mul;
                    }
                    PLLSrc::HSE(prediv) => {
                        rcc.cfgr0.modify(|_, w| unsafe {
                            w.pllmul()
                                .bits(mul as u8)
                                .pllxtpre()
                                .bit(prediv.to_taw())
                                .pllsrc()
                                .bit(false)
                        });
                        clocks.sysclk = clocks.hse.unwrap() / prediv * mul;
                    }
                }
                clocks.pllclk = Some(clocks.sysclk);

                // Enable PLL
                rcc.ctlr.modify(|_, w| w.pllon().set_bit());
                // Wait for PLL to stabilize
                while rcc.ctlr.read().pllrdy().bit_is_clear() {}

                rcc.cfgr0.modify(|_, w| w.sw().pll());
                while !rcc.cfgr0.read().sws().is_pll() {}
            }
        }

        clocks.hclk = clocks.sysclk / self.ahb_pre;
        clocks.pclk1 = clocks.hclk / self.apb1_pre;
        clocks.pclk2 = clocks.hclk / self.apb2_pre;

        // FLASH access clock frequency cannot be more than 72 MHz.
        if clocks.sysclk.to_Hz() > 72_000_000 {
            // = sysclk/2
            flash.ctlr.modify(|_, w| w.sckmode().clear_bit())
        } else {
            // = sysclk
            flash.ctlr.modify(|_, w| w.sckmode().set_bit())
        }

        /*

        // Configure LSE if provided
        if let Some(lse) = self.lse.as_ref() {
            // Configure the LSE mode
            match lse.mode {
                LSEClockMode::Bypass => rcc.bdctlr.modify(|_, w| w.lsebyp().set_bit()),
                LSEClockMode::Oscillator => rcc.bdctlr.modify(|_, w| w.lsebyp().clear_bit()),
            }
            // Enable the LSE.
            rcc.bdctlr.modify(|_, w| w.lseon().set_bit());
            // Wait for the LSE to stabilise.
            while rcc.bdctlr.read().lserdy().bit_is_clear() {}
        }

        if self.lsi.is_some() {
            rcc.rstsckr.modify(|_, w| w.lsion().set_bit());
            while rcc.rstsckr.read().lsirdy().bit_is_clear() {}
        }

        // other PLL here

        rcc.cfgr0.modify(|_, w| w.mco().variant(self.mco.into()));

        */

        unsafe {
            riscv::asm::delay(16);
        }

        clocks
    }
}

// LSI / LSE

/// LSE clock mode.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LSEClockMode {
    /// Enable LSE oscillator to use external crystal or ceramic resonator.
    Crystal,
    /// Bypass LSE oscillator to use external clock source.
    /// Use this if an external oscillator is used which is not connected to `OSC32_IN` such as a MEMS resonator.
    Bypass,
}

/// LSE Clock.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LSEClock {
    /// Input frequency.
    freq: Hertz,
    /// Mode.
    mode: LSEClockMode,
}

impl LSEClock {
    /// Provide LSE frequency.
    pub fn new(mode: LSEClockMode) -> Self {
        // Sets the LSE clock source to 32.768 kHz.
        LSEClock {
            freq: 32_768.Hz(),
            mode,
        }
    }
}

/// Microcontroller clock output
///
/// Value on reset: None
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MCO {
    /// No clock
    None,
    /// System clock selected
    Sysclk,
    /// HSI clock selected
    Hsi,
    /// HSE oscillator clock selected
    Hse,
    /// PLL/2 clock selected
    PllDiv2,
    // For CH32F20x_D8C、CH32V30x_D8C
    /*
    /// PLL2 clock selected
    Pll2,
    /// PLL3/2 clock selected
    Pll3Div2,
    /// XT1 external 3-25 MHz oscillator clock selected
    Xt1,
    /// PLL3 clock selected
    Pll3,
    */
}

impl From<MCO> for crate::pac::rcc::cfgr0::MCO_A {
    fn from(mco: MCO) -> Self {
        match mco {
            MCO::None => crate::pac::rcc::cfgr0::MCO_A::NoMco,
            MCO::Sysclk => crate::pac::rcc::cfgr0::MCO_A::Sysclk,
            MCO::Hsi => crate::pac::rcc::cfgr0::MCO_A::Hsi,
            MCO::Hse => crate::pac::rcc::cfgr0::MCO_A::Hse,
            MCO::PllDiv2 => crate::pac::rcc::cfgr0::MCO_A::PllDiv2,
        }
    }
}

/// Configure the "mandatory" clocks (`sysclk`, `hclk`, `pclk1` and `pclk2')
/// and return them via the `Clocks` struct.
///
/// The user shouldn't call freeze more than once as the clocks parameters
/// cannot be changed after the clocks have started.
///
/// The implementation makes the following choice: HSI is always chosen over
/// HSE except when HSE is provided. When HSE is provided, HSE is used
/// wherever it is possible.
//#define SYSCLK_FREQ_HSE    HSE_VALUE
//#define SYSCLK_FREQ_48MHz_HSE  48000000
//#define SYSCLK_FREQ_56MHz_HSE  56000000
//#define SYSCLK_FREQ_72MHz_HSE  72000000
//#define SYSCLK_FREQ_96MHz_HSE  96000000
//#define SYSCLK_FREQ_120MHz_HSE  120000000
//#define SYSCLK_FREQ_144MHz_HSE  144000000
//#define SYSCLK_FREQ_HSI    HSI_VALUE
//#define SYSCLK_FREQ_48MHz_HSI  48000000
//#define SYSCLK_FREQ_56MHz_HSI  56000000
//#define SYSCLK_FREQ_72MHz_HSI  72000000
//#define SYSCLK_FREQ_96MHz_HSI  96000000
//#define SYSCLK_FREQ_120MHz_HSI  120000000
//#define SYSCLK_FREQ_144MHz_HSI  144000000

/// Frozen clock frequencies
///
/// The existence of this value indicates that the clock configuration can no longer be changed
#[derive(Clone, Copy, Debug)]
pub struct Clocks {
    pub sysclk: Hertz,
    pub hclk: Hertz,
    pub pclk1: Hertz,
    pub pclk2: Hertz,
    pub pllclk: Option<Hertz>,
    pub hse: Option<Hertz>,
    pub lse: Option<Hertz>,
    pub lsi: Option<Hertz>,
}

impl Clocks {
    /// Returns the frequency of the AHB1
    pub fn hclk(&self) -> Hertz {
        self.hclk
    }

    /// Returns the frequency of the APB1
    pub fn pclk1(&self) -> Hertz {
        self.pclk1
    }

    /// Returns the frequency of the APB2
    pub fn pclk2(&self) -> Hertz {
        self.pclk2
    }

    /// Returns the system (core) frequency
    pub fn sysclk(&self) -> Hertz {
        self.sysclk
    }

    /// Returns the frequency of the `HSE` if `Some`, else `None`.
    pub fn hse(&self) -> Option<Hertz> {
        self.hse
    }

    /// Returns the frequency of the `LSE` if `Some`, else `None`.
    pub fn lse(&self) -> Option<Hertz> {
        self.lse
    }

    /// Returns the frequency of the `LSI` if `Some`, else `None`.
    pub fn lsi(&self) -> Option<Hertz> {
        self.lsi
    }
}

impl Default for Clocks {
    fn default() -> Self {
        Clocks {
            sysclk: 8.MHz(),
            hclk: 8.MHz(),
            pclk1: 8.MHz(),
            pclk2: 8.MHz(),
            pllclk: None,
            hse: None,
            lse: None,
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
        clocks.pclk1
    }
}

impl BusClock for APB2 {
    fn clock(clocks: &Clocks) -> Hertz {
        clocks.pclk2
    }
}

impl BusTimerClock for APB1 {
    fn timer_clock(clocks: &Clocks) -> Hertz {
        if clocks.hclk == clocks.pclk1 {
            // APB1 prescaler=1
            clocks.pclk1
        } else {
            clocks.pclk1 * 2
        }
    }
}

impl BusTimerClock for APB2 {
    fn timer_clock(clocks: &Clocks) -> Hertz {
        if clocks.hclk == clocks.pclk2 {
            // APB2 prescaler=1
            clocks.pclk2
        } else {
            clocks.pclk2 * 2
        }
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
