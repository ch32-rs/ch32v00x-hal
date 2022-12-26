//! Reset and clock control.

use core::cmp::min;
use core::mem;

mod enable;

use fugit::{HertzU32 as Hertz, RateExtU32};

use crate::pac::{rcc, FLASH, PWR, RCC};

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
            cfgr: CFGR {
                hse: None,
                hclk: None,
                sysclk: None,
                pclk1: None,
                pclk2: None,
                lse: None,
                lsi: None,
                use_pll: false,
                pllmul: unsafe { mem::transmute(0b0000_u8) },
                pllxtpre: false,
                mco: MCO::None,
            },
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
    pub cfgr: CFGR,
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

/// HSE clock mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HSEClockMode {
    /// Enable HSE oscillator to use external crystal or ceramic resonator.
    Oscillator,
    /// Bypass HSE oscillator to use external clock source.
    Bypass,
}

/// HSE Clock.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HSEClock {
    /// Input frequency.
    pub(crate) freq: Hertz,
    /// Mode.
    mode: HSEClockMode,
}

impl HSEClock {
    /// Provide HSE frequency.
    ///
    /// # Panics
    ///
    /// Panics if the frequency is outside the valid range. The frequency must be between
    /// 3 MHz and 25 MHz in oscillator mode and between 1 MHz and 25 MHz in bypass mode.
    // TODO: 32Mhz for CH32V203RB
    pub fn new(freq: Hertz, mode: HSEClockMode) -> Self {
        let valid_range = match mode {
            // Source: 3.3.2 High-speed clock (HSI/HSE)
            HSEClockMode::Oscillator => Hertz::MHz(3)..=Hertz::MHz(25),
            HSEClockMode::Bypass => Hertz::MHz(1)..=Hertz::MHz(25),
        };
        assert!(valid_range.contains(&freq));

        HSEClock { freq, mode }
    }
}

/// LSE clock mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LSEClockMode {
    /// Enable LSE oscillator to use external crystal or ceramic resonator.
    Oscillator,
    /// Bypass LSE oscillator to use external clock source.
    /// Use this if an external oscillator is used which is not connected to `OSC32_IN` such as a MEMS resonator.
    Bypass,
}

/// LSE Clock.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

/// PLLMUL multiplication factors.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PLLMUL {
    #[cfg(not(feature = "d8c"))]
    Mul2 = 0b0000,
    #[cfg(feature = "d8c")]
    Mul18 = 0b0000,
    Mul3 = 0b0001,
    Mul4 = 0b0010,
    Mul5 = 0b0011,
    Mul6 = 0b0100,
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
    Mul6_5 = 0b1101,
    #[cfg(feature = "d8c")]
    Mul15 = 0b1110,
    #[cfg(feature = "d8c")]
    Mul16 = 0b1111,
}

impl PLLMUL {
    fn mul(&self, pll: u64) -> u64 {
        match self {
            #[cfg(not(feature = "d8c"))]
            PLLMUL::Mul2 => pll * 2,
            PLLMUL::Mul3 => pll * 3,
            PLLMUL::Mul4 => pll * 4,
            PLLMUL::Mul5 => pll * 5,
            PLLMUL::Mul6 => pll * 6,
            PLLMUL::Mul7 => pll * 7,
            PLLMUL::Mul8 => pll * 8,
            PLLMUL::Mul9 => pll * 9,
            PLLMUL::Mul10 => pll * 10,
            PLLMUL::Mul11 => pll * 11,
            PLLMUL::Mul12 => pll * 12,
            PLLMUL::Mul13 => pll * 13,
            PLLMUL::Mul14 => pll * 14,
            PLLMUL::Mul15 => pll * 15,
            PLLMUL::Mul16 => pll * 16,
            PLLMUL::Mul18 => pll * 18,
            #[cfg(feature = "d8c")]
            PLLMUL::Mul6_5 => pll / 2 * 13,
        }
    }
}

/// Microcontroller clock output
///
/// Value on reset: None
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
enum PllSource {
    HSI = 0, // HSI/2
    HSE = 1, // HSE or HSE/2
}

impl Default for PllSource {
    fn default() -> Self {
        PllSource::HSI
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
enum PllXtPre {
    Div1 = 0,
    Div2 = 1,
    // for CH32F20x_D8W、CH32V20x_D8、CH32V20x_D8W
    // Div4 = 0,
    // Div8 = 1,
}

impl Default for PllXtPre {
    fn default() -> Self {
        PllXtPre::Div1
    }
}

impl PllXtPre {
    fn to_raw(&self) -> bool {
        *self == PllXtPre::Div2
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
struct InternalRCCConfig {
    // 1, 2, 4, 8, 16, 64, 128, 256, 512
    hpre: u8,
    // 1, 2, 4, 8, 16
    ppre1: u8,
    ppre2: u8,
    // flash_waitstates: u8,
    // overdrive: bool,
    // vos_scale: VOSscale,
    pllsrc: PllSource,
    pllxtpre: PllXtPre,
}

/// Clock configuration register.
#[derive(Debug, PartialEq, Eq)]
pub struct CFGR {
    hse: Option<HSEClock>,
    hclk: Option<u32>,
    sysclk: Option<u32>,
    pclk1: Option<u32>,
    pclk2: Option<u32>,
    lse: Option<LSEClock>,
    lsi: Option<Hertz>,
    use_pll: bool,
    pllxtpre: bool,
    pllmul: PLLMUL,
    mco: MCO,
}

impl CFGR {
    /// Configures the HSE oscillator.
    pub fn hse(mut self, hse: HSEClock) -> Self {
        self.hse = Some(hse);
        self
    }

    /// Sets HCLK frequency.
    ///
    /// The HCLK is used for the AHB bus, core, memory and DMA.
    ///
    /// # Panics
    ///
    /// Panics if the frequency is larger than 144 MHz.
    pub fn hclk(mut self, freq: Hertz) -> Self {
        assert!(freq.raw() <= 144_000_000);

        self.hclk = Some(freq.raw());
        self
    }

    /// Sets the SYSCLK frequency.
    ///
    /// This sets the SYSCLK frequency and sets up the USB clock if defined.
    /// The provided frequency must be between 3 Mhz and 144 Mhz.
    /// When the USB interface is used, the frequency of CPU must be 48MHz or 96MHz or 144MHz.
    ///
    /// # Panics
    ///
    /// Panics if the frequency is not between 3 MHz and 144 MHz.
    pub fn sysclk(mut self, sysclk: Hertz) -> Self {
        assert!((3_000_000..=144_000_000).contains(&sysclk.raw()));

        self.sysclk = Some(sysclk.raw());
        self
    }

    /// Sets the PCLK1 clock (APB1 clock).
    ///
    /// If this method isn't called the maximum allowed frequency is used for PCLK1.
    ///
    /// # Panics
    ///
    /// Panics if the frequency is not between 12.5 MHz and 54 MHz.
    pub fn pclk1(mut self, freq: Hertz) -> Self {
        assert!((3_000_000..=144_000_000).contains(&freq.raw()));

        self.pclk1 = Some(freq.raw());
        self
    }

    /// Sets PCLK2 clock (APB2 clock).
    ///
    /// If this method isn't called the maximum allowed frequency is used for PCLK2.
    ///
    /// # Panics
    ///
    /// Panics if the frequency is not between 3 MHz and 144 MHz.
    pub fn pclk2(mut self, freq: Hertz) -> Self {
        assert!((3_000_000..=144_000_000).contains(&freq.raw()));

        self.pclk2 = Some(freq.raw());
        self
    }

    /// Sets the LSE clock source to 32.768 kHz.
    pub fn lse(mut self, lse: LSEClock) -> Self {
        self.lse = Some(lse);
        self
    }

    /// Sets the LSI clock source to 40 kHz.
    ///
    // FIXME: CH32V20x_D8W、CH32F20x_D8W has LSI32K
    pub fn lsi(mut self) -> Self {
        self.lsi = Some(40.kHz());
        self
    }

    /// Sets the SYSCLK clock source to the main PLL.
    pub fn use_pll(mut self) -> Self {
        self.use_pll = true;
        self
    }

    /// Sets the PLL multiplication factor for the main PLL.
    pub fn pllmul(mut self, pllmul: PLLMUL) -> Self {
        self.pllmul = self.pllmul;
        self
    }

    /// Sets the MCO source
    pub fn mco(mut self, mco: MCO) -> Self {
        self.mco = mco;
        self
    }

    /// Output clock calculation
    fn calculate_clocks(&self) -> (Clocks, InternalRCCConfig) {
        let mut config = InternalRCCConfig::default();

        let base_clk = u64::from(
            match self.hse.as_ref() {
                Some(hse) => hse.freq,
                None => HSI_FREQUENCY,
            }
            .raw(),
        );

        let mut sysclk = base_clk;

        // TODO: handle /4 /8 div
        if self.use_pll {
            sysclk = if self.hse.is_some() {
                if self.pllxtpre {
                    self.pllmul.mul(base_clk as u64 / 2)
                } else {
                    self.pllmul.mul(base_clk as u64)
                }
            } else {
                // HSI/2
                self.pllmul.mul(base_clk as u64 / 2)
            };
        }

        // SYSCLK, must be <= 216 Mhz. By default, HSI/HSE frequency is chosen
        assert!(sysclk <= 144_000_000);
        let sysclk = sysclk as u32;

        // HCLK. By default, SYSCLK frequency is chosen. Because of the method
        // of clock multiplication and division, even if `sysclk` is set to be
        // the same as `hclk`, it can be slightly inferior to `sysclk` after
        // pllm, pllp... calculations
        let hclk: u32 = min(sysclk, self.hclk.unwrap_or(sysclk));
        let prediv = sysclk / hclk;
        assert!(sysclk % hclk == 0); // hclk must be a divisor of sysclk
        config.hpre = match prediv {
            1 => 0b0000,
            2 => 0b1000,
            4 => 0b1001,
            8 => 0b1010,
            16 => 0b1011,
            64 => 0b1100,
            128 => 0b1101,
            256 => 0b1110,
            512 => 0b1111,
            _ => unreachable!(),
        };

        let max_pclk = 144_000_000;
        let pclk1: u32 = min(max_pclk, self.pclk1.unwrap_or(hclk));
        let pclk2: u32 = min(max_pclk, self.pclk2.unwrap_or(hclk));

        // Configure PPRE1
        let ppre1_val = hclk / pclk1;
        assert!(hclk % pclk1 == 0); // pclk1 must be a divisor of hclk
        config.ppre1 = match ppre1_val {
            1 => 0b000,
            2 => 0b100,
            4 => 0b101,
            8 => 0b110,
            16 => 0b111,
            _ => unreachable!(),
        };

        // Configure PPRE2
        let ppre2_val = hclk / pclk2;
        assert!(hclk % pclk2 == 0); // pclk2 must be a divisor of hclk
        config.ppre2 = match ppre2_val {
            1 => 0b000,
            2 => 0b100,
            4 => 0b101,
            8 => 0b110,
            16 => 0b111,
            _ => unreachable!(),
        };

        let clocks = Clocks {
            hclk: hclk.Hz(),
            pclk1: pclk1.Hz(),
            pclk2: pclk2.Hz(),
            sysclk: sysclk.Hz(),
            hse: self.hse.map(|hse| hse.freq),
            lse: self.lse.map(|lse| lse.freq),
            lsi: self.lsi,
        };

        (clocks, config)
    }

    fn pll_configure(&mut self) {
        // handle PLLXTPRE PLLSRC PLLMUL
        let base_clk = match self.hse.as_ref() {
            Some(hse) => hse.freq,
            None => HSI_FREQUENCY,
        }
        .raw();

        let sysclk = if let Some(clk) = self.sysclk {
            clk
        } else {
            base_clk
        };

        if base_clk == sysclk {
            self.use_pll = false;
            return;
        }

        // check if PLLXTPRE and PLLMUL allow to obtain the requested Sysclk,
        // so that we don't have to calculate them
        if (sysclk as u64) == self.pllmul.mul(base_clk as u64) {
            return;
        }

        #[cfg(any(feature = "ch32v203rb", feature = "ch32v208"))]
        unimplemented!();

        if (sysclk as u64) == self.pllmul.mul(base_clk as u64 / 2) {
            self.pllxtpre = true;
            return;
        }

        // now calculate pllmul
        // unimplemented!()
    }

    /// Configures the default clock settings.
    ///
    /// Set SYSCLK as 144 Mhz and setup USB clock if defined.
    pub fn set_defaults(self) -> Self {
        self.sysclk(72.MHz())
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
    pub fn freeze(mut self) -> Clocks {
        let flash = unsafe { &(*FLASH::ptr()) };
        let rcc = unsafe { &(*RCC::ptr()) };
        let pwr = unsafe { &(*PWR::ptr()) };

        self.pll_configure();

        let (clocks, config) = self.calculate_clocks();

        // Turn on HSI
        rcc.ctlr.modify(|_, w| w.hsion().set_bit());
        while rcc.ctlr.read().hsirdy().bit_is_clear() {}
        // Switch to HSI
        rcc.cfgr0.modify(|_, w| w.sw().hsi());

        // Configure HSE if provided
        if let Some(hse) = self.hse {
            // Configure the HSE mode
            match self.hse.as_ref().unwrap().mode {
                HSEClockMode::Bypass => rcc.ctlr.modify(|_, w| w.hsebyp().set_bit()),
                HSEClockMode::Oscillator => rcc.ctlr.modify(|_, w| w.hsebyp().clear_bit()),
            }
            // Start HSE
            rcc.ctlr.modify(|_, w| w.hseon().set_bit());
            while rcc.ctlr.read().hserdy().bit_is_clear() {}
        }

        if self.use_pll {
            // Disable PLL, PLLMUL, PLLXTPRE, PLLSRC can only be written when PLL is off.
            rcc.ctlr.modify(|_, w| w.pllon().clear_bit());

            rcc.cfgr0.modify(|_, w| unsafe {
                w.pllmul()
                    .bits(self.pllmul as u8)
                    .pllxtpre()
                    .variant(config.pllxtpre.to_raw())
                    .pllsrc()
                    .bit(self.hse.is_some())
            });

            // Enable PLL
            rcc.ctlr.modify(|_, w| w.pllon().set_bit());
            // Wait for PLL to stabilise
            while rcc.ctlr.read().pllrdy().bit_is_clear() {}
        }

        // Enable PWR domain
        rcc.apb1pcenr.modify(|_, w| w.pwren().set_bit());
        // Enable editing backup_domain RCC_BDCTLR
        pwr.ctlr.modify(|_, w| w.dbp().set_bit());

        // Configure LSE if provided
        if self.lse.is_some() {
            // Configure the LSE mode
            match self.lse.as_ref().unwrap().mode {
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

        rcc.cfgr0
            .modify(|_, w| unsafe { w.mco().variant(self.mco.into()) });

        // FLASH access clock frequency cannot be more than 72 MHz.
        // if clocks.sysclk.to_Hz() * 2 < 72_000_000 {
        //    flash.ctlr.modify(|_, w| w.sckmode().set_bit())
        // }

        // Configure HCLK, PCLK1, PCLK2
        rcc.cfgr0.modify(|_, w| unsafe {
            w.ppre1()
                .bits(config.ppre1)
                .ppre2()
                .bits(config.ppre2)
                .hpre()
                .bits(config.hpre)
        });

        // Select SYSCLK source
        if self.use_pll {
            rcc.cfgr0.modify(|_, w| w.sw().pll());
            while !rcc.cfgr0.read().sws().is_pll() {}
        } else if self.hse.is_some() {
            rcc.cfgr0.modify(|_, w| w.sw().hse());
            while !rcc.cfgr0.read().sws().is_hse() {}
        } else {
            rcc.cfgr0.modify(|_, w| w.sw().hsi());
            while !rcc.cfgr0.read().sws().is_hsi() {}
        }

        unsafe {
            riscv::asm::delay(16);
        }

        clocks
    }
}

/// Frozen clock frequencies
///
/// The existence of this value indicates that the clock configuration can no longer be changed
#[derive(Clone, Copy, Debug)]
pub struct Clocks {
    hclk: Hertz,
    pclk1: Hertz,
    pclk2: Hertz,
    sysclk: Hertz,
    hse: Option<Hertz>,
    lse: Option<Hertz>,
    lsi: Option<Hertz>,
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
