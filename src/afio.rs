use ch32v0::ch32v003::AFIO;

use crate::rcc::{Enable, Rcc, Reset};

pub trait AfioExt {
    /// Enable the Alternate Function IO peripheral
    fn configure(self, rcc: &mut Rcc) -> Afio;
}

impl AfioExt for AFIO {
    fn configure(self, rcc: &mut Rcc) -> Afio {
        // Power on and reset the AFIO peripheral
        AFIO::enable(&mut rcc.apb2);
        AFIO::reset(&mut rcc.apb2);

        Afio { afio: self }
    }
}

/// Alternate function IO controller
///
/// This is generally used to configure pin remappings for other peripherals
pub struct Afio {
    afio: AFIO,
}

impl Afio {
    /// Configure the I2C1REMAP1 and I2C1_RM bits
    #[inline]
    pub(crate) fn set_i2c1_remap(&mut self, (high, low): (bool, bool)) {
        self.afio
            .pcfr
            .write(|w| w.i2c1remap1().bit(high).i2c1rm().bit(low));
    }
}
