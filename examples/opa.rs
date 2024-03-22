//! This example enables the in-built operational amplifier of the CH32V003.
#![no_std]
#![no_main]

use panic_halt as _;

use ch32v00x_hal as hal;
use ch32v00x_hal::prelude::*;

#[qingke_rt::entry]
fn main() -> ! {
    let p = ch32v0::ch32v003::Peripherals::take().unwrap();

    let mut rcc = p.RCC.constrain();
    let _clocks = rcc.config.freeze();

    let gpioa = p.GPIOA.split(&mut rcc);
    let gpiod = p.GPIOD.split(&mut rcc);

    // Op-amp non-inverting input.
    let opa_p = gpioa.pa2;
    // PD7 is configured as NRST by default.
    // let opa_p = gpiod.pd7;

    // Op-amp inverting input.
    let opa_n = gpioa.pa1;
    // let opa_n = gpiod.pd0;

    // Op-amp output.
    let opa_o = gpiod.pd4;

    #[allow(unused)]
    let opa = hal::extend::opa::Opa::enable(opa_p, opa_n, opa_o);

    // Pins are available for other uses after disabling opa.
    // let (opa_p, opa_n, opa_o) = opa.disable();

    loop {}
}
