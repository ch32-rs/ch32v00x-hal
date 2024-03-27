//! Power Control (PWR)

use crate::{
    pac::PWR,
    rcc::{Clocks, Enable, Rcc},
};

pub enum PVDVoltageThreshold {
    Rising2_85Falling2_7 = 0b000,
    Rising3_05Falling2_9 = 0b001,
    Rising3_3Falling3_15 = 0b010,
    Rising3_5Falling3_3 = 0b011,
    Rising3_7Falling3_5 = 0b100,
    Rising3_9Falling3_7 = 0b101,
    Rising4_1Falling3_9 = 0b110,
    Rising4_4Falling4_2 = 0b111,
}

pub struct Pwr {
    pwr: PWR,
    _clocks: Clocks,
}

impl Pwr {
    pub fn pwr(pwr: PWR, rcc: &mut Rcc, _clocks: Clocks) -> Self {
        PWR::enable(&mut rcc.apb1);

        Self { pwr, _clocks }
    }

    /// set threshold voltage for pvd
    pub fn pvd_threshold_voltage(&mut self, threshold: PVDVoltageThreshold) {
        self.pwr
            .ctlr
            .modify(|_, w| w.pls().variant(threshold as u8));
    }

    /// enable pvd
    pub fn enable_pvd(&mut self) {
        self.pwr.ctlr.modify(|_, w| w.pvde().set_bit());
    }

    /// disable pvd
    pub fn disable_pvd(&mut self) {
        self.pwr.ctlr.modify(|_, w| w.pvde().clear_bit());
    }

    /// pvd output: returns 1 if vdd is above threshold voltage
    pub fn pvd_output(&mut self) -> bool {
        self.pwr.csr.read().pvdo().bit_is_clear()
    }
}
