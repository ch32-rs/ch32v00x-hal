//! In-built operation amplifier control.

use crate::{
    gpio::{Floating, Input, PA1, PA2, PD0, PD4, PD7},
    pac,
};

/// In-built operational amplifier control.
pub struct Opa<P: OPA_P, N: OPA_N> {
    opa_p: P,
    opa_n: N,
    opa_o: PD4<Input<Floating>>,
}

impl<P: OPA_P, N: OPA_N> Opa<P, N> {
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
    pub fn enable(opa_p: P, opa_n: N, opa_o: PD4<Input<Floating>>) -> Self {
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
    pub fn disable(self) -> (P, N, PD4) {
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
pub trait OPA_P {
    /// Value of `OPA_NSEL` bit of [`EXTEND_CTR`](pac::EXTEND) to select this pin.
    const OPA_NSEL: bool;
}
/// `OPP0` - `PA2`
impl OPA_P for PA2<Input<Floating>> {
    const OPA_NSEL: bool = false;
}
/// `OPP1` - `PD7`
impl OPA_P for PD7<Input<Floating>> {
    const OPA_NSEL: bool = true;
}

/// Pins that can be used as the inverting input to the operation amplifier impl this trait.
pub trait OPA_N {
    /// Value of `OPA_PSEL` bit of [`EXTEND_CTR`](pac::EXTEND) to select this pin.
    const OPA_PSEL: bool;
}
/// `OPN0` - `PA1`
impl OPA_N for PA1<Input<Floating>> {
    const OPA_PSEL: bool = false;
}
/// `OPN1` - `PD0`
impl OPA_N for PD0<Input<Floating>> {
    const OPA_PSEL: bool = true;
}
