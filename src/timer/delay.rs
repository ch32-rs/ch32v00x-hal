//! Delays
use crate::{pac::SYSTICK};
use core::ops::{Deref, DerefMut};
use fugit::{MicrosDurationU32, TimerDurationU32};

const SYSTICK_RANGE: u32 = 0x8000_0000;

/// Timer as a delay provider (SYSTICKick by default)
pub struct SysDelay {
    pub systick: SYSTICK,
    scale: u32,
}

impl SysDelay {
    pub fn new(systick: SYSTICK, scale: u32) -> Self {
        systick.ctlr.write(|w| w.stre().set_bit().stclk().set_bit().ste().set_bit());
        unsafe {systick.cmpr.write(|w| w.cmp().bits(SYSTICK_RANGE - 1))};
        SysDelay {
            systick, scale
        }
    }

    pub fn delay(&mut self, us: u32) {
        // The SysTick Reload Value register supports values between 1 and 0x7FFFFFFF.
        // Here less than maximum is used so we have some play if there's a long running interrupt.
        const MAX_RVR: u32 = SYSTICK_RANGE / 2 - 1;

        let mut total_rvr = us * self.scale;


        while total_rvr != 0 {
            let current_rvr = total_rvr.min(MAX_RVR);

            let start_rvr = self.systick.cnt.read().cnt().bits();
            // Update the tracking variable while we are waiting...
            total_rvr -= current_rvr;
            // Use the wrapping substraction and the modulo to deal with the systick wrapping around
            while (self.systick.cnt.read().cnt().bits().wrapping_sub(start_rvr)) % SYSTICK_RANGE < current_rvr {}
        }
    }
}
