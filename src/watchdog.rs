//! Watchdog peripherals

use crate::{ pac::IWDG, time::MilliSeconds };
use fugit::ExtU32;

/// Wraps the Independent Watchdog (IWDG) peripheral
pub struct IndependentWatchdog {
    iwdg: IWDG,
}

const LSI_KHZ: u32 = 128;
const MAX_PR: u8 = 8;
const MAX_RL: u16 = 0xFFF;
const KR_ACCESS: u16 = 0x5555;
const KR_RELOAD: u16 = 0xAAAA;
const KR_START: u16 = 0xCCCC;

impl IndependentWatchdog {
    /// Wrap and start the watchdog
    pub fn new(iwdg: IWDG) -> Self {
        IndependentWatchdog { iwdg }
    }
    fn setup(&self, timeout_ms: u32) {
        let mut pr = 0;
        while pr < MAX_PR && Self::timeout_period(pr, MAX_RL) < timeout_ms {
            pr += 1;
        }

        let max_period = Self::timeout_period(pr, MAX_RL);
        let max_rl = u32::from(MAX_RL);
        let rl = (timeout_ms * max_rl / max_period).min(max_rl) as u16;

        self.access_registers(|iwdg| {
            iwdg.pscr.modify(|_, w| unsafe { w.pr().bits(pr) });
            iwdg.rldr.modify(|_, w| unsafe { w.rl().bits(rl) });
        });
    }

    fn is_pr_updating(&self) -> bool {
        self.iwdg.statr.read().pvu().bit()
    }

    /// Returns the interval in ms
    pub fn interval(&self) -> MilliSeconds {
        while self.is_pr_updating() {}

        let pr = self.iwdg.pscr.read().pr().bits();
        let rl = self.iwdg.rldr.read().rl().bits();
        let ms = Self::timeout_period(pr, rl);
        ms.millis()
    }

    /// pr: Prescaler divider bits, rl: reload value
    ///
    /// Returns ms
    fn timeout_period(pr: u8, rl: u16) -> u32 {
        let divider: u32 = match pr {
            0b000 => 4,
            0b001 => 8,
            0b010 => 16,
            0b011 => 32,
            0b100 => 64,
            0b101 => 128,
            0b110 => 256,
            0b111 => 256,
            _ => return 0,
        };
        (u32::from(rl) + 1) * divider / LSI_KHZ
    }

    fn access_registers<A, F: FnMut(&IWDG) -> A>(&self, mut f: F) -> A {
        // Unprotect write access to registers
        self.iwdg.ctlr.write(|w| unsafe { w.key().bits(KR_ACCESS) });
        let a = f(&self.iwdg);

        // Protect again
        self.iwdg.ctlr.write(|w| unsafe { w.key().bits(KR_RELOAD) });
        a
    }

    pub fn start(&mut self, period: MilliSeconds) {
        self.setup(period.ticks());

        self.iwdg.ctlr.write(|w| unsafe { w.key().bits(KR_START) });
    }

    pub fn feed(&mut self) {
        self.iwdg.ctlr.write(|w| unsafe { w.key().bits(KR_RELOAD) });
    }
}
