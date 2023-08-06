use fugit::{HertzU32, RateExtU32};

use crate::{
    afio::Afio,
    gpio::*,
    pac::{
        i2c1::{star1, star2},
        I2C1,
    },
    rcc::{BusClock, Clocks, Enable, Rcc, Reset},
};

/// Ready to use I2C peripheral
pub struct I2c<Scl, Sda> {
    i2c: I2C1,
    scl: Scl,
    sda: Sda,
}

/// I2C low/high duty cycle when using Fast Mode (> 100kHz)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DutyCycle {
    /// 33%
    Perc33,
    /// 36%
    Perc36,
}

/// I2C peripheral configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct I2cConfig {
    pub speed: HertzU32,
    pub duty: DutyCycle,
}

impl I2cConfig {
    /// 100 kbit/s 33% duty cycle
    #[inline]
    pub const fn slow_mode() -> I2cConfig {
        Self {
            speed: HertzU32::kHz(100),
            duty: DutyCycle::Perc33,
        }
    }

    /// 400 kbit/s 33% duty cycle
    #[inline]
    pub const fn fast_mode() -> I2cConfig {
        Self {
            speed: HertzU32::kHz(400),
            duty: DutyCycle::Perc33,
        }
    }

    /// 1 mbit/s 33% duty cycle
    #[inline]
    pub const fn fast_mode_plus() -> I2cConfig {
        Self {
            speed: HertzU32::kHz(1000),
            duty: DutyCycle::Perc33,
        }
    }
}

/// 400kHz/33%
impl Default for I2cConfig {
    fn default() -> Self {
        Self::fast_mode()
    }
}

impl<Scl, Sda> I2c<Scl, Sda>
where
    (Scl, Sda): I2C1Pair,
{
    /// Initialise the I2C1 peripheral with valid SCL and SDA pins
    pub fn i2c1(
        i2c: I2C1,
        scl: Scl,
        sda: Sda,
        config: I2cConfig,
        afio: &mut Afio,
        rcc: &mut Rcc,
        clocks: &Clocks,
    ) -> Self {
        // Ensure i2c is enabled and reset to known state
        I2C1::enable(&mut rcc.apb1);
        I2C1::reset(&mut rcc.apb1);

        // Reset peripheral state, just to be safe?
        i2c.ctlr1.modify(|_, w| w.swrst().set_bit());
        i2c.ctlr1.modify(|_, w| w.swrst().clear_bit());

        // Configure the remap bits in AFIO to match our pin selection
        afio.set_i2c1_remap(<(Scl, Sda)>::REMAP_BITS);

        // Configure peripheral clock (valid range 2-36mhz)
        let freq = I2C1::clock(clocks).to_MHz().clamp(2, 36);
        i2c.ctlr2.modify(|_, w| w.freq().variant(freq as u8));

        let fast_mode = config.speed > 100u32.kHz::<1, 1>();
        let speed = config.speed.to_Hz();

        // Calculate bus speed. The source of these values is a bit obscure?
        let ccr = match (fast_mode, config.duty) {
            (false, _) => I2C1::clock(clocks).to_Hz() / (speed * 2),
            (true, DutyCycle::Perc33) => I2C1::clock(clocks).to_Hz() / (speed * 3),
            (true, DutyCycle::Perc36) => I2C1::clock(clocks).to_Hz() / (speed * 25),
        };

        // Set clock flags
        i2c.ckcfgr.modify(|_, w| {
            w.ccr() // Clock rate
                .variant(ccr as u16)
                .f_s() // Fast mode
                .bit(fast_mode)
                .duty() // Duty cycle
                .bit(config.duty == DutyCycle::Perc36)
        });

        // Start peripheral and enable acknowledgements
        i2c.ctlr1.modify(|_, w| w.pe().set_bit());
        i2c.ctlr1.modify(|_, w| w.ack().set_bit());

        Self { i2c, scl, sda }
    }

    /// Deconstruct the I2C peripheral and return it's raw hardware resources
    pub fn release(self) -> (I2C1, Scl, Sda) {
        // Disable the peripheral
        self.i2c.ctlr1.modify(|_, w| w.pe().clear_bit());

        (self.i2c, self.scl, self.sda)
    }

    #[inline]
    fn wait_while(&self, f: impl Fn(star1::R, star2::R) -> bool) {
        while {
            // // It is important to read STAR1 before STAR2
            let s1 = self.i2c.star1.read();
            let s2 = self.i2c.star2.read();
            f(s1, s2)
        } {}
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    BusError,
    AcknowledgeFailure,
    ArbitrationLost,
    Overrun,
}

impl<Scl, Sda> embedded_hal::blocking::i2c::Write for I2c<Scl, Sda>
where
    (Scl, Sda): I2C1Pair,
{
    type Error = Error;

    #[inline(never)]
    fn write(&mut self, address: u8, bytes: &[u8]) -> Result<(), Self::Error> {
        // Wait till idle
        self.wait_while(|_, s2| s2.busy().bit_is_set());

        // Send start event and take control of the bus
        self.i2c.ctlr1.modify(|_, w| w.start().set_bit());

        // Wait till start has been sent and master mode is assigned
        self.wait_while(|s1, s2| {
            s1.sb().bit_is_clear() || s2.busy().bit_is_clear() || s2.msl().bit_is_clear()
        });

        // Send address + write flag
        self.i2c.datar.write(|w| w.datar().variant(address << 1));

        // Wait address is till sent
        self.wait_while(|s1, s2| {
            s1.addr().bit_is_clear()
                || s1.tx_e().bit_is_clear()
                || s2.busy().bit_is_clear()
                || s2.msl().bit_is_clear()
                || s2.tra().bit_is_clear()
        });

        // Send each byte one by one
        for byte in bytes {
            self.wait_while(|a, _| a.tx_e().bit_is_clear());
            self.i2c.datar.write(|w| w.datar().variant(*byte));
        }

        // Wait for whole transmission to complete
        self.wait_while(|s1, s2| {
            s1.btf().bit_is_clear()
                || s1.tx_e().bit_is_clear()
                || s2.busy().bit_is_clear()
                || s2.msl().bit_is_clear()
                || s2.tra().bit_is_clear()
        });

        // Stop transmission
        self.i2c.ctlr1.modify(|_, w| w.stop().set_bit());

        // Check error codes
        let s1 = self.i2c.star1.read();
        if s1.berr().bit() {
            return Err(Error::BusError);
        } else if s1.af().bit() {
            return Err(Error::AcknowledgeFailure);
        } else if s1.arlo().bit() {
            return Err(Error::ArbitrationLost);
        } else if s1.ovr().bit() {
            return Err(Error::Overrun);
        }

        Ok(())
    }
}

/// Marker trait for valid combinations of SCL and SDA for multiplexed I2C pins
pub trait I2C1Pair {
    /// High and Low bits of remap register (I2C1REMAP1 and I2C1_RM)
    // TODO: Should this just be u8? Does it matter?
    const REMAP_BITS: (bool, bool);
}

/// Default pin remapping option (0b00)
/// # T and U
/// While Open Drain is recommended, pins can be used in Push-pull configuration as well
impl<T, U> I2C1Pair for (PC2<Alternate<T>>, PC1<Alternate<U>>) {
    const REMAP_BITS: (bool, bool) = (false, false);
}

/// Pin remapping option 2 (0b01)
/// # T and U
/// While Open Drain is recommended, pins can be used in Push-pull configuration as well
impl<T, U> I2C1Pair for (PD1<Alternate<T>>, PD0<Alternate<U>>) {
    const REMAP_BITS: (bool, bool) = (false, true);
}

/// Pin remapping option 3 (0b1X)
/// # T and U
/// While Open Drain is recommended, pins can be used in Push-pull configuration as well
impl<T, U> I2C1Pair for (PC5<Alternate<T>>, PC6<Alternate<U>>) {
    const REMAP_BITS: (bool, bool) = (true, false);
}
