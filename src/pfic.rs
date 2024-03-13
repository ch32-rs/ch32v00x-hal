//! Programmable Fast Interrupt Controller (PFIC)

use crate::{
    pac::{rcc, AFIO, PFIC},
    rcc::Enable,
};

pub trait PficExt {
    fn constrain(self) -> Pfic;
}

impl PficExt for PFIC {
    fn constrain(self) -> Pfic {
        Pfic { pfic: self }
    }
}

pub struct Pfic {
    pfic: PFIC,
}
