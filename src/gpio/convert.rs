//! Conversion between pin modes

use super::*;

impl<const P: char, const N: u8, MODE> Pin<P, N, MODE> {
    pub(super) fn set_alternate(&mut self) {
        let offset = (4 * N) % 32;
        let cfgr = 0b1011; // Alternative PushPull, Output 50MHz
        unsafe {
            if N >= 8 {
                (*Gpio::<P>::ptr())
                    .cfghr
                    .modify(|r, w| w.bits((r.bits() & !(0b1111 << offset)) | (cfgr << offset)));
            } else {
                (*Gpio::<P>::ptr())
                    .cfglr
                    .modify(|r, w| w.bits((r.bits() & !(0b1111 << offset)) | (cfgr << offset)));
            }
        }
    }

    /// Configures the pin to operate alternate mode
    pub fn into_alternate(mut self) -> Pin<P, N, Alternate<PushPull>> {
        self.set_alternate();
        Pin::new()
    }

    /// Configures the pin to operate in alternate open drain mode
    pub fn into_alternate_open_drain(self) -> Pin<P, N, Alternate<OpenDrain>> {
        self.into_alternate().set_open_drain()
    }

    /// Configures the pin to operate as a floating input pin
    pub fn into_floating_input(mut self) -> Pin<P, N, Input<Floating>> {
        self.mode::<Input<Floating>>();
        Pin::new()
    }

    /// Configures the pin to operate as a pulled down input pin
    pub fn into_pull_down_input(mut self) -> Pin<P, N, Input<PullDown>> {
        self.mode::<Input<PullDown>>();
        Pin::new()
    }

    /// Configures the pin to operate as a pulled up input pin
    pub fn into_pull_up_input(mut self) -> Pin<P, N, Input<PullUp>> {
        self.mode::<Input<PullUp>>();
        Pin::new()
    }

    /// Configures the pin to operate as an open drain output pin
    /// Initial state will be low.
    pub fn into_open_drain_output(mut self) -> Pin<P, N, Output<OpenDrain>> {
        self.mode::<Output<OpenDrain>>();
        Pin::new()
    }

    /// Configures the pin to operate as an open-drain output pin.
    /// `initial_state` specifies whether the pin should be initially high or low.
    pub fn into_open_drain_output_in_state(
        mut self,
        initial_state: PinState,
    ) -> Pin<P, N, Output<OpenDrain>> {
        self._set_state(initial_state);
        self.mode::<Output<OpenDrain>>();
        Pin::new()
    }

    /// Configures the pin to operate as an push pull output pin
    /// Initial state will be low.
    pub fn into_push_pull_output(mut self) -> Pin<P, N, Output<PushPull>> {
        self._set_low();
        self.mode::<Output<PushPull>>();
        Pin::new()
    }

    /// Configures the pin to operate as an push-pull output pin.
    /// `initial_state` specifies whether the pin should be initially high or low.
    pub fn into_push_pull_output_in_state(
        mut self,
        initial_state: PinState,
    ) -> Pin<P, N, Output<PushPull>> {
        self._set_state(initial_state);
        self.mode::<Output<PushPull>>();
        Pin::new()
    }

    /// Configures the pin to operate as an analog input pin
    pub fn into_analog(mut self) -> Pin<P, N, Analog> {
        self.mode::<Analog>();
        Pin::new()
    }

    // TODO: into_dynamic

    /// Puts `self` into mode `M`.
    ///
    /// This violates the type state constraints from `MODE`, so callers must
    /// ensure they use this properly.
    #[inline(always)]
    pub(super) fn mode<M: PinMode>(&mut self) {
        let offset = (4 * N) % 32;
        let cfgr = (M::CNFR << 2) | M::MODER;
        unsafe {
            if N >= 8 {
                (*Gpio::<P>::ptr())
                    .cfghr
                    .modify(|r, w| w.bits((r.bits() & !(0b1111 << offset)) | (cfgr << offset)));
            } else {
                (*Gpio::<P>::ptr())
                    .cfglr
                    .modify(|r, w| w.bits((r.bits() & !(0b1111 << offset)) | (cfgr << offset)));
            }
            if let Some(odr) = M::ODR {
                (*Gpio::<P>::ptr())
                    .outdr
                    .modify(|r, w| w.bits((r.bits() & !(1 << N)) | ((odr as u32) << N)));
            }
        }
    }
}

// TODO: with_mode

/// Marker trait for valid pin modes (type state).
///
/// It can not be implemented by outside types.
pub trait PinMode: crate::Sealed {
    // These constants are used to implement the pin configuration code.
    // They are not part of public API.

    /// Input / Output mode
    #[doc(hidden)]
    const MODER: u32;

    /// Actual pin mode
    #[doc(hidden)]
    const CNFR: u32;

    /// Push up or push down
    #[doc(hidden)]
    const ODR: Option<bool> = None;
}

impl crate::Sealed for Input<Floating> {}
impl PinMode for Input<Floating> {
    const MODER: u32 = 0b00;
    const CNFR: u32 = 0b01;
}

impl crate::Sealed for Input<PullDown> {}
impl PinMode for Input<PullDown> {
    const MODER: u32 = 0b00;
    const CNFR: u32 = 0b10;
    const ODR: Option<bool> = Some(false);
}

impl crate::Sealed for Input<PullUp> {}
impl PinMode for Input<PullUp> {
    const MODER: u32 = 0b00;
    const CNFR: u32 = 0b10;
    const ODR: Option<bool> = Some(true);
}

// Analog in
impl crate::Sealed for Analog {}
impl PinMode for Analog {
    const MODER: u32 = 0b00;
    const CNFR: u32 = 0b00;
}

impl crate::Sealed for Output<OpenDrain> {}
impl PinMode for Output<OpenDrain> {
    const MODER: u32 = 0b11;
    const CNFR: u32 = 0b01;
}

impl crate::Sealed for Output<PushPull> {}
impl PinMode for Output<PushPull> {
    const MODER: u32 = 0b11;
    const CNFR: u32 = 0b00;
}
