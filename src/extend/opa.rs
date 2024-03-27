//! In-built operation amplifier control.
//!
//! The OPA peripheral does not have programmable gain - it relies on external feedback resistors/connections.

use crate::{
    gpio::{Analog, Input, PA1, PA2, PD0, PD4, PD7},
    pac, Sealed,
};

/// In-built operational amplifier control.
pub struct Opa<MODE, P: OpaP, N: OpaN> {
    opa_p: P,
    opa_n: N,
    opa_o: PD4<MODE>,
}

impl<MODE: ValidMode, P: OpaP, N: OpaN> Opa<MODE, P, N> {
    /// Enable the OPA, taking hold of the pins the OPA is using until disabled.
    ///
    /// Pins that can be passed for non-inverting input - `opa_p`:
    /// * `PA2`
    /// * `PD7`
    ///
    /// Pins that can be passed for inverting input - `opa_n`:
    /// * `PA1`
    /// * `PD0`
    ///
    /// The output of the amplifier is always `PD4`.
    pub fn enable(opa_p: P, opa_n: N, opa_o: PD4<MODE>) -> Self {
        unsafe {
            (*pac::EXTEND::ptr()).extend_ctr.modify(|_, w| {
                w.opa_en()
                    .set_bit()
                    .opa_psel()
                    .bit(P::OPA_NSEL)
                    .opa_nsel()
                    .bit(N::OPA_PSEL)
            });
        }

        // We hold on to the pins until the OPA is disabled.
        Opa {
            opa_p,
            opa_n,
            opa_o,
        }
    }

    /// Turn off the OPA peripheral, returning the pins it was using.
    pub fn disable(self) -> (P, N, PD4<MODE>) {
        unsafe {
            // Clearing all bits back to reset value of 0.
            (*pac::EXTEND::ptr()).extend_ctr.modify(|_, w| {
                w.opa_en()
                    .clear_bit()
                    .opa_psel()
                    .clear_bit()
                    .opa_nsel()
                    .clear_bit()
            });
        }

        (self.opa_p, self.opa_n, self.opa_o)
    }
}

/// Pins that can be used as the non-inverting input to the operation amplifier impl this trait.
pub trait OpaP: Sealed {
    /// Value of `OPA_NSEL` bit of [`EXTEND_CTR`](pac::EXTEND) to select this pin.
    const OPA_NSEL: bool;
}
/// `OPP0` - `PA2`
impl<MODE: ValidMode> OpaP for PA2<MODE> {
    const OPA_NSEL: bool = false;
}
/// `OPP1` - `PD7`
impl<MODE: ValidMode> OpaP for PD7<MODE> {
    const OPA_NSEL: bool = true;
}

/// Pins that can be used as the inverting input to the operation amplifier impl this trait.
pub trait OpaN: Sealed {
    /// Value of `OPA_PSEL` bit of [`EXTEND_CTR`](pac::EXTEND) to select this pin.
    const OPA_PSEL: bool;
}
/// `OPN0` - `PA1`
impl<MODE: ValidMode> OpaN for PA1<MODE> {
    const OPA_PSEL: bool = false;
}
/// `OPN1` - `PD0`
impl<MODE: ValidMode> OpaN for PD0<MODE> {
    const OPA_PSEL: bool = true;
}

impl<T> Sealed for PD0<T> {}
impl<T> Sealed for PA1<T> {}
impl<T> Sealed for PD7<T> {}
impl<T> Sealed for PA2<T> {}

/// Pin modes implementing this are pin modes which are suitable to use with OPA.
pub trait ValidMode {}

/// It seems reasonable that a GPIO which is also being measured
/// by the ADC can be used with OPA. (Untested.)
impl ValidMode for Analog {}

/// It seems reasonable that a GPIO which is an input, even if
/// it has a pull-resistor enabled, can still be used with OPA. (Untested.)
impl<MODE> ValidMode for Input<MODE> {}
