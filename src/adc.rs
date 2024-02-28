use embedded_hal_02::adc::{Channel, OneShot};
use crate::gpio::{self, Analog};
use crate::pac;
use crate::rcc::{self, Clocks, Enable, Reset};
use qingke::riscv::asm::delay;
use fugit::HertzU32;

/// Continuous mode
pub struct Continuous;
/// Scan mode
pub struct Scan;

/// ADC configuration
pub struct Adc<'a, ADC> {
    rb: ADC,
    sample_time: SampleTime,
    align: Align,
    clocks: &'a Clocks,
}

/// ADC sampling time
///
/// Options for the sampling time, each is T + 0.5 ADC clock cycles.
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SampleTime {
    /// 1.5 cycles sampling time
    T_1,
    /// 7.5 cycles sampling time
    T_7,
    /// 13.5 cycles sampling time
    T_13,
    /// 28.5 cycles sampling time
    T_28,
    /// 41.5 cycles sampling time
    T_41,
    /// 55.5 cycles sampling time
    T_55,
    /// 71.5 cycles sampling time
    T_71,
    /// 239.5 cycles sampling time
    T_239,
}

impl Default for SampleTime {
    /// Get the default sample time (currently 28.5 cycles)
    fn default() -> Self {
        SampleTime::T_28
    }
}

impl From<SampleTime> for u8 {
    fn from(val: SampleTime) -> Self {
        use SampleTime::*;
        match val {
            T_1 => 0,
            T_7 => 1,
            T_13 => 2,
            T_28 => 3,
            T_41 => 4,
            T_55 => 5,
            T_71 => 6,
            T_239 => 7,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// ADC data register alignment
pub enum Align {
    /// Right alignment of output data
    Right,
    /// Left alignment of output data
    Left,
}

impl Default for Align {
    /// Default: right alignment
    fn default() -> Self {
        Align::Right
    }
}

impl From<Align> for bool {
    fn from(val: Align) -> Self {
        match val {
            Align::Right => false,
            Align::Left => true,
        }
    }
}

macro_rules! adc_pins {
    ($ADC:ty, $($pin:ty => $chan:expr),+ $(,)*) => {
        $(
            impl Channel<$ADC> for $pin {
                type ID = u8;

                fn channel() -> u8 { $chan }
            }
        )+
    };
}

adc_pins!(pac::ADC1,
    gpio::PA2<Analog> => 0_u8,
    gpio::PA1<Analog> => 1_u8,
    gpio::PC4<Analog> => 2_u8,
    gpio::PD2<Analog> => 3_u8,
    gpio::PD3<Analog> => 4_u8,
    gpio::PD5<Analog> => 5_u8,
    gpio::PD6<Analog> => 6_u8,
    gpio::PD4<Analog> => 7_u8,
);

/// Stored ADC config can be restored using the `Adc::restore_cfg` method
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub struct StoredConfig(SampleTime, Align);

impl<'a> Adc<'a, pac::ADC1> {
    /// Init a new Adc
    ///
    /// Sets all configurable parameters to one-shot defaults,
    /// performs a boot-time calibration.
    pub fn new(adc: pac::ADC1, clocks: &'a Clocks) -> Self {
        let mut s = Self {
            rb: adc,
            sample_time: SampleTime::default(),
            align: Align::default(),
            clocks,
        };
        s.enable_clock();
        s.power_down();
        s.reset();
        s.setup_oneshot();
        s.power_up();

        // The manual states that we need to wait two ADC clocks cycles after power-up
        // before starting calibration, we already delayed in the power-up process, but
        // if the adc clock is too low that was not enough.
        if s.clocks.adcclk() < HertzU32::kHz(2500) {
            let two_adc_cycles = s.clocks.sysclk() / s.clocks.adcclk() * 2;
            let already_delayed = s.clocks.sysclk() / HertzU32::kHz(800);
            if two_adc_cycles > already_delayed {
                unsafe { delay(two_adc_cycles - already_delayed) };
            }
        }
        s.calibrate();
        s
    }

    /// Save current ADC config
    pub fn save_cfg(&mut self) -> StoredConfig {
        StoredConfig(self.sample_time, self.align)
    }

    /// Restore saved ADC config
    pub fn restore_cfg(&mut self, cfg: StoredConfig) {
        self.sample_time = cfg.0;
        self.align = cfg.1;
    }

    /// Reset the ADC config to default, return existing config
    pub fn default_cfg(&mut self) -> StoredConfig {
        let cfg = self.save_cfg();
        self.sample_time = SampleTime::default();
        self.align = Align::default();
        cfg
    }

    /// Set ADC sampling time
    ///
    /// Options can be found in [SampleTime](crate::adc::SampleTime).
    pub fn set_sample_time(&mut self, t_samp: SampleTime) {
        self.sample_time = t_samp;
    }

    /// Set the Adc result alignment
    ///
    /// Options can be found in [Align](crate::adc::Align).
    pub fn set_align(&mut self, align: Align) {
        self.align = align;
    }

    /// Returns the largest possible sample value for the current settings
    pub fn max_sample(&self) -> u16 {
        match self.align {
            Align::Left => u16::max_value(),
            Align::Right => (1 << 10) - 1,
        }
    }
    #[inline(always)]
    pub fn set_external_trigger(&mut self, trigger: u8) {
        self.rb.ctlr2.modify(|_, w| w.extsel().variant(trigger))
    }
    fn power_up(&mut self) {
        self.rb.ctlr2.modify(|_, w| w.adon().set_bit());

        // The reference manual says that a stabilization time is needed after power_up,
        // this time can be found in the datasheets.
        // Here we are delaying for approximately 1us, considering 1.25 instructions per
        // cycle. Do we support a chip which needs more than 1us ?
        unsafe { delay(self.clocks.sysclk() / HertzU32::kHz(800)) };
    }

    fn power_down(&mut self) {
        self.rb.ctlr2.modify(|_, w| w.adon().clear_bit());
    }

    fn reset(&mut self) {
		let mut apb2 = rcc::APB2::new();
        <pac::ADC1>::reset(&mut apb2);
    }

    fn enable_clock(&mut self) {
		let mut apb2 = rcc::APB2::new();
        <pac::ADC1>::enable(&mut apb2);
    }

    fn disable_clock(&mut self) {
		let mut apb2 = rcc::APB2::new();
        <pac::ADC1>::disable(&mut apb2);
    }

    fn calibrate(&mut self) {
        /* reset calibration */
        self.rb.ctlr2.modify(|_, w| w.rstcal().set_bit());
        while self.rb.ctlr2.read().rstcal().bit_is_set() {}

        /* calibrate */
        self.rb.ctlr2.modify(|_, w| w.cal().set_bit());
        while self.rb.ctlr2.read().cal().bit_is_set() {}
    }

    fn setup_oneshot(&mut self) {
        self.rb.ctlr2.modify(|_, w| {
            unsafe { w.cont()
                .clear_bit()
                .exttrig()
                .set_bit()
                .extsel()
                .bits(0b111) }
        });
        self.rb
            .ctlr1
            .modify(|_, w| w.scan().clear_bit().discen().set_bit());
        self.rb.rsqr1.modify(|_, w| unsafe { w.l().bits(0b0) });
    }
    fn set_channel_sample_time(&mut self, chan: u8, sample_time: SampleTime) {
        let sample_time = sample_time.into();
		unsafe {
			match chan {
				0 => self.rb.samptr2_charge2.modify(|_, w| w.smp0_tkcg0().bits(sample_time) ),
				1 => self.rb.samptr2_charge2.modify(|_, w| w.smp1_tkcg1().bits(sample_time) ),
				2 => self.rb.samptr2_charge2.modify(|_, w| w.smp2_tkcg2().bits(sample_time) ),
				3 => self.rb.samptr2_charge2.modify(|_, w| w.smp3_tkcg3().bits(sample_time) ),
				4 => self.rb.samptr2_charge2.modify(|_, w| w.smp4_tkcg4().bits(sample_time) ),
				5 => self.rb.samptr2_charge2.modify(|_, w| w.smp5_tkcg5().bits(sample_time) ),
				6 => self.rb.samptr2_charge2.modify(|_, w| w.smp6_tkcg6().bits(sample_time) ),
				7 => self.rb.samptr2_charge2.modify(|_, w| w.smp7_tkcg7().bits(sample_time) ),
				8 => self.rb.samptr2_charge2.modify(|_, w| w.smp8_tkcg8().bits(sample_time) ),
				9 => self.rb.samptr2_charge2.modify(|_, w| w.smp9_tkcg9().bits(sample_time) ),
				_ => unreachable!(),
			}
		}
    }
    fn set_regular_sequence(&mut self, channels: &[u8]) {
        let len = channels.len();
        let bits = channels
            .iter()
            .take(6)
            .enumerate()
            .fold(0u32, |s, (i, c)| s | ((*c as u32) << (i * 5)));
        self.rb.rsqr3.write(|w| unsafe { w.bits(bits) });
        if len > 6 {
            let bits = channels
                .iter()
                .skip(6)
                .take(6)
                .enumerate()
                .fold(0u32, |s, (i, c)| s | ((*c as u32) << (i * 5)));
            self.rb.rsqr2.write(|w| unsafe { w.bits(bits) });
        }
        if len > 12 {
            let bits = channels
                .iter()
                .skip(12)
                .take(4)
                .enumerate()
                .fold(0u32, |s, (i, c)| s | ((*c as u32) << (i * 5)));
            self.rb.rsqr1.write(|w| unsafe { w.bits(bits) });
        }
        self.rb.rsqr1.modify(|_, w| unsafe { w.l().bits((len - 1) as u8) });
    }

    fn set_continuous_mode(&mut self, continuous: bool) {
        self.rb.ctlr2.modify(|_, w| w.cont().bit(continuous));
    }

    fn set_discontinuous_mode(&mut self, channels_count: Option<u8>) {
        self.rb.ctlr1.modify(|_, w| match channels_count {
            Some(count) => unsafe { w.discen().set_bit().discnum().bits(count) },
            None => w.discen().clear_bit(),
        });
    }
    /**
      Performs an ADC conversion

      NOTE: Conversions can be started by writing a 1 to the ADON
      bit in the `CR2` while it is already 1, and no other bits
      are being written in the same operation. This means that
      the EOC bit *might* be set already when entering this function
      which can cause a read of stale values

      The check for `ctlr2.swstart.bit_is_set` *should* fix it, but
      does not. Therefore, ensure you do not do any no-op modifications
      to `ctlr2` just before calling this function
    */
    pub fn convert(&mut self, chan: u8) -> u16 {
        // Dummy read in case something accidentally triggered
        // a conversion by writing to CR2 without changing any
        // of the bits
        self.rb.rdatar.read().data().bits();

        self.set_channel_sample_time(chan, self.sample_time);
        self.rb.rsqr3.modify(|_, w| unsafe { w.sq1().bits(chan) });
        // ADC start conversion of regular sequence
        self.rb
            .ctlr2
            .modify(|_, w| w.swstart().set_bit().align().bit(self.align.into()));
        while self.rb.ctlr2.read().swstart().bit_is_set() {}
        // ADC wait for conversion results
        while self.rb.statr.read().eoc().bit_is_clear() {}

        let res = self.rb.rdatar.read().data().bits();
        res as u16
    }
    /// Powers down the ADC, disables the ADC clock and releases the ADC Peripheral
    pub fn release(mut self) -> pac::ADC1 {
        self.power_down();
        self.disable_clock();
        self.rb
    }
}

impl<'a> Adc<'a, pac::ADC1> {
    fn read_aux(&mut self, chan: u8) -> u16 {
		/* ADC TSPD mask */
		const CTLR2_TSVREFE_SET: u32   = 0x00800000;
		const CTLR2_TSVREFE_RESET: u32 = 0xFF7FFFFF;
        let tsv_off = if (self.rb.ctlr2.read().bits() & CTLR2_TSVREFE_SET) > 0 {
            self.rb.ctlr2.modify(|r, w| unsafe { w.bits(r.bits() | CTLR2_TSVREFE_SET) } );

            // The reference manual says that a stabilization time is needed after the powering the
            // sensor, this time can be found in the datasheets.
            // Here we are delaying for approximately 10us, considering 1.25 instructions per
            // cycle. Do we support a chip which needs more than 10us ?
            unsafe { delay(self.clocks.sysclk().raw() / 80_000) };
            true
        } else {
            false
        };

        let val = self.convert(chan);

        if tsv_off {
            self.rb.ctlr2.modify(|r, w| unsafe { w.bits(r.bits() & CTLR2_TSVREFE_RESET) });
        }

        val
    }

    pub fn read_vcal(&mut self) -> u16 {
        self.read_aux(9u8)
    }

    pub fn read_vref(&mut self) -> u16 {
        self.read_aux(8u8)
    }
}

pub trait ChannelTimeSequence {
    /// Set ADC sampling time for particular channel
    fn set_channel_sample_time(&mut self, chan: u8, sample_time: SampleTime);
    /// ADC Set a Regular Channel Conversion Sequence
    ///
    /// Define a sequence of channels to be converted as a regular group.
    fn set_regular_sequence(&mut self, channels: &[u8]);
    /// Set ADC continuous conversion
    ///
    /// When continuous conversion is enabled conversion does not stop at the last selected group channel but continues again from the first selected group channel.
    fn set_continuous_mode(&mut self, continuous: bool);
    /// Set ADC discontinuous mode
    ///
    /// It can be used to convert a short sequence of conversions (up to 8) which is a part of the regular sequence of conversions.
    fn set_discontinuous_mode(&mut self, channels_count: Option<u8>);
}

impl<'a> ChannelTimeSequence for Adc<'a, pac::ADC1> {
	#[inline(always)]
	fn set_channel_sample_time(&mut self, chan: u8, sample_time: SampleTime) {
		self.set_channel_sample_time(chan, sample_time);
	}
	#[inline(always)]
	fn set_regular_sequence (&mut self, channels: &[u8]) {
		self.set_regular_sequence(channels);
	}
	#[inline(always)]
	fn set_continuous_mode(&mut self, continuous: bool) {
		self.set_continuous_mode(continuous);
	}
	#[inline(always)]
	fn set_discontinuous_mode(&mut self, channels: Option<u8>) {
		self.set_discontinuous_mode(channels);
	}
}

impl<'a, WORD, PIN> OneShot<pac::ADC1, WORD, PIN> for Adc<'a, pac::ADC1>
where
    WORD: From<u16>,
    PIN: Channel<pac::ADC1, ID = u8>,
{
    type Error = ();
    fn read(&mut self, _pin: &mut PIN) -> nb::Result<WORD, Self::Error> {
        let res = self.convert(PIN::channel());
        Ok(res.into())
    }
}
