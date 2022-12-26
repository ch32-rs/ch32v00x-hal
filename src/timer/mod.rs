use crate::pac::SYSTICK;
use crate::rcc::{self, Clocks};
use fugit::HertzU32 as Hertz;

pub mod counter;
pub use counter::*;
pub mod delay;
pub use delay::*;

/// Timer wrapper
pub struct Timer<TIM> {
    pub(crate) tim: TIM,
    pub(crate) clk: Hertz,
}

/// Interrupt events
#[derive(Clone, Copy, PartialEq)]
pub enum SysEvent {
    /// [Timer] timed out / count down ended
    Update,
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum Error {
    /// Timer is disabled
    Disabled,
    WrongAutoReload,
}

pub trait SysTimerExt: Sized {
    /// Creates timer which takes [Hertz] as Duration
    fn counter_hz(self, clocks: &Clocks) -> SysCounterHz;

    /// Creates timer with custom precision (core frequency recommended is known)
    fn counter<const FREQ: u32>(self, clocks: &Clocks) -> SysCounter<FREQ>;
    /// Creates timer with precision of 1 μs (1 MHz sampling)
    fn counter_us(self, clocks: &Clocks) -> SysCounterUs {
        self.counter::<1_000_000>(clocks)
    }
    /// Blocking [Delay] with custom precision
    fn delay(self, clocks: &Clocks) -> SysDelay;
}

impl SysTimerExt for SYSTICK {
    fn counter_hz(self, clocks: &Clocks) -> SysCounterHz {
        Timer::systick(self, clocks).counter_hz()
    }
    fn counter<const FREQ: u32>(self, clocks: &Clocks) -> SysCounter<FREQ> {
        Timer::systick(self, clocks).counter()
    }
    fn delay(self, clocks: &Clocks) -> SysDelay {
        Timer::systick_external(self, clocks).delay()
    }
}

impl Timer<SYSTICK> {
    /// Initialize SysTick timer
    pub fn systick(mut tim: SYSTICK, clocks: &Clocks) -> Self {
        tim.set_clock_source(SystickClkSource::Core);
        Self {
            tim,
            clk: clocks.hclk(),
        }
    }

    /// Initialize SysTick timer and set it frequency to `HCLK / 8`
    pub fn systick_external(mut tim: SYSTICK, clocks: &Clocks) -> Self {
        tim.set_clock_source(SystickClkSource::External);
        Self {
            tim,
            clk: clocks.hclk() / 8,
        }
    }

    pub fn configure(&mut self, clocks: &Clocks) {
        self.tim.set_clock_source(SystickClkSource::Core);
        self.clk = clocks.hclk();
    }

    pub fn configure_external(&mut self, clocks: &Clocks) {
        self.tim.set_clock_source(SystickClkSource::External);
        self.clk = clocks.hclk() / 8;
    }

    pub fn release(self) -> SYSTICK {
        self.tim
    }

    /// Starts listening for an `event`
    pub fn listen(&mut self, event: SysEvent) {
        match event {
            SysEvent::Update => self.tim.enable_interrupt(),
        }
    }

    /// Stops listening for an `event`
    pub fn unlisten(&mut self, event: SysEvent) {
        match event {
            SysEvent::Update => self.tim.disable_interrupt(),
        }
    }
}

pub enum SystickClkSource {
    // HCLK
    Core,
    // HCLK / 8
    External,
}

pub(crate) trait SystickExt {
    fn clear_current(&mut self);
    fn disable_counter(&mut self);
    fn disable_interrupt(&mut self);
    fn enable_counter(&mut self);
    fn enable_interrupt(&mut self);
    fn get_clock_source(&mut self) -> SystickClkSource;
    fn get_current() -> u64;
    fn get_reload() -> u64;
    fn has_wrapped(&mut self) -> bool;
    fn is_counter_enabled(&mut self) -> bool;
    fn is_interrupt_enabled(&mut self) -> bool;
    fn set_clock_source(&mut self, clk_source: SystickClkSource);
    fn set_reload(&mut self, value: u64);
}

impl SystickExt for SYSTICK {
    fn clear_current(&mut self) {
        // self.cnth.write(|w| unsafe { w.bits(0) });
        // self.cntl.write(|w| unsafe { w.bits(0) });
        self.ctlr.modify(|_, w| w.init().set_bit());
    }

    fn disable_counter(&mut self) {
        self.ctlr.modify(|_, w| w.ste().clear_bit());
    }

    fn disable_interrupt(&mut self) {
        self.ctlr.modify(|_, w| w.stie().clear_bit());
    }

    fn enable_counter(&mut self) {
        self.ctlr.modify(|_, w| w.ste().set_bit());
    }

    fn enable_interrupt(&mut self) {
        self.ctlr.modify(|_, w| w.stie().set_bit());
    }

    fn get_clock_source(&mut self) -> SystickClkSource {
        if self.ctlr.read().stclk().bit_is_set() {
            SystickClkSource::Core
        } else {
            SystickClkSource::External // HCLK/8
        }
    }

    fn get_current() -> u64 {
        unsafe { (*Self::PTR).cnt().read().bits() }
    }

    fn get_reload() -> u64 {
        unsafe { (*Self::PTR).cmp().read().bits() }
    }

    fn has_wrapped(&mut self) -> bool {
        self.sr.read().cntif().bit_is_set()
    }

    fn is_counter_enabled(&mut self) -> bool {
        self.ctlr.read().ste().bit_is_set()
    }

    fn is_interrupt_enabled(&mut self) -> bool {
        self.ctlr.read().stie().bit_is_set()
    }

    fn set_clock_source(&mut self, clk_source: SystickClkSource) {
        match clk_source {
            SystickClkSource::Core => self.ctlr.modify(|_, w| w.stclk().set_bit()),
            SystickClkSource::External => self.ctlr.modify(|_, w| w.stclk().clear_bit()),
        }
    }

    fn set_reload(&mut self, value: u64) {
        self.cmp().write(|w| unsafe { w.bits(value) });
    }
}
