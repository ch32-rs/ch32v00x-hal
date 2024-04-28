use crate::pac;

pub trait CPin<REMAP, const C: u8> {}
pub struct Ch<const C: u8>;
pub const C1: u8 = 0;
pub const C2: u8 = 1;
pub const C3: u8 = 2;
pub const C4: u8 = 3;

pub(crate) mod sealed {
    pub trait Remap {
        type Periph;
        const REMAP: u8;

        fn remap();
    }
}

macro_rules! remap {
    ($($name:ident: ($TIMX:ty, $state:literal, $P1:ident, $P2:ident, $P3:ident, $P4:ident, { $remapex:expr }),)+) => {
        $(
            pub struct $name;
            impl sealed::Remap for $name {
                type Periph = $TIMX;
                const REMAP: u8 = $state;

                fn remap() {
                    let afio = unsafe { &(*pac::AFIO::ptr()) };
                    afio.pcfr.modify($remapex);
                }
            }
            impl<MODE> CPin<$name, 0> for crate::gpio::$P1<MODE> {}
            impl<MODE> CPin<$name, 1> for crate::gpio::$P2<MODE> {}
            impl<MODE> CPin<$name, 2> for crate::gpio::$P3<MODE> {}
            impl<MODE> CPin<$name, 3> for crate::gpio::$P4<MODE> {}
        )+
    }
}

remap!(
    Tim1NoRemap: (pac::TIM1, 0b00, PD2, PA1, PC3, PC4, {|_, w| unsafe { w.tim1rm().bits(Self::REMAP)}}),
    Tim1PartialRemap1: (pac::TIM1, 0b01, PC6, PC7, PC0, PD3, {|_, w| unsafe { w.tim1rm().bits(Self::REMAP)}}),
    Tim1PartialRemap2: (pac::TIM1, 0b10, PD2, PA1, PC3, PC4, {|_, w| unsafe { w.tim1rm().bits(Self::REMAP)}}),
    Tim1FullRemap: (pac::TIM1, 0b11, PC4, PC7, PC5, PD4, {|_, w| unsafe { w.tim1rm().bits(Self::REMAP)}}),
);

remap!(
    Tim2NoRemap: (pac::TIM2, 0b00, PD4, PD3, PC0, PD7, {|_, w| unsafe { w.tim2rm().bits(Self::REMAP)}}),
    Tim2PartialRemap1: (pac::TIM2, 0b01, PC5, PC2, PD2, PC1, {|_, w| unsafe { w.tim2rm().bits(Self::REMAP)}}),
    Tim2PartialRemap2: (pac::TIM2, 0b10, PC1, PD3, PC0, PD7, {|_, w| unsafe { w.tim2rm().bits(Self::REMAP)}}),
    Tim2FullRemap: (pac::TIM2, 0b11, PC1, PC7, PD6, PD5, {|_, w| unsafe { w.tim2rm().bits(Self::REMAP)}}),
);
