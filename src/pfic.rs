//! Programmable Fast Interrupt Controller (PFIC)

use ch32v0::ch32v003::AFIO;

use crate::{pac::{rcc, PFIC}, rcc::Enable};

pub trait PficExt {
    fn constrain(self) -> Pfic;
}

impl PficExt for PFIC {
    fn constrain(self) -> Pfic {
        Pfic {
            pfic: self,
        }
    }
}

pub struct Pfic {
    pfic: PFIC,
}
