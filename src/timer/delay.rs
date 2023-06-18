//! Delays
use crate::pac::SYSTICK;
use crate::rcc::Clocks;

/// A delay using the systick timer
///
/// Timing mostly doesn't vary from interrupts
pub struct SysDelay {
    systick: SYSTICK,
    scale: u32,
    // The SysTick counter has a width of 32 bits and will be configured to run freely.
    // We'll only wait half the theoretically possible time to have some leeway in case of interrupts.
    max_us: u32
}

impl SysDelay {
    pub fn new(systick: SYSTICK, clocks: &Clocks) -> Self {
        systick.ctlr.write(|w| w.stclk().set_bit().ste().set_bit());
        let scale = clocks.hclk().to_MHz() as u32;
        SysDelay {
            systick,
            scale,
            max_us: (8000_0000u32 / scale) - 1,
        }
    }
}

impl embedded_hal_alpha::delay::DelayUs for SysDelay {
    fn delay_us(&mut self, mut us: u32) {
        // Scale the us inside the loop, to avoid overflow scenarios
        while us != 0 {
            let current_us = us.min(self.max_us);
            let current_rvr = current_us * self.scale;

            let start_rvr = self.systick.cnt.read().cnt().bits();
            // Update the tracking variable while we are waiting...
            us -= current_us;
            // Use the wrapping substraction to deal with the systick wrapping around
            while (self.systick.cnt.read().cnt().bits().wrapping_sub(start_rvr)) < current_rvr {}
        }
    }
}

impl embedded_hal::blocking::delay::DelayUs<u32> for SysDelay {
    fn delay_us(&mut self, us: u32) {
        embedded_hal_alpha::delay::DelayUs::delay_us(self, us as _);
    }
}

impl embedded_hal::blocking::delay::DelayUs<u16> for SysDelay {
    fn delay_us(&mut self, us: u16) {
        embedded_hal_alpha::delay::DelayUs::delay_us(self, us as _);
    }
}

impl embedded_hal::blocking::delay::DelayUs<u8> for SysDelay {
    fn delay_us(&mut self, us: u8) {
        embedded_hal_alpha::delay::DelayUs::delay_us(self, us as _);
    }
}

impl embedded_hal::blocking::delay::DelayMs<u32> for SysDelay {
    // Multiplying ms so we can call delay_us directly might overflow, so implement an outer loop
    fn delay_ms(&mut self, mut ms: u32) {
        const MAX_MS: u32 = 0x0010_0000;
        while ms != 0 {
            let current_ms = if ms <= MAX_MS { ms } else { MAX_MS };
            embedded_hal::blocking::delay::DelayUs::delay_us(self, current_ms as u32 * 1_000);
            ms -= current_ms;
        }
    }
}

impl embedded_hal::blocking::delay::DelayMs<u16> for SysDelay {
    fn delay_ms(&mut self, ms: u16) {
        // Call delay_us directly, so we don't have to use the additional
        // delay loop the u32 variant uses
        embedded_hal::blocking::delay::DelayUs::delay_us(self, ms as u32 * 1_000);
    }
}

impl embedded_hal::blocking::delay::DelayMs<u8> for SysDelay {
    fn delay_ms(&mut self, ms: u8) {
        self.delay_ms(ms as u16);
    }
}
