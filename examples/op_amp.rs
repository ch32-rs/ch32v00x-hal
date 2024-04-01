//! This example enables the in-built operational amplifier of the CH32V003.
//!
//! The OPA peripheral does not have programmable gain - it relies on external feedback resistors/connections.
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
    let non_inverting_pin = gpioa.pa2;
    // PD7 is configured as NRST by default.
    // let non_inverting_pin = gpiod.pd7;

    // Op-amp inverting input.
    let inverting_pin = gpioa.pa1;
    // let inverting_pin = gpiod.pd0;

    // Op-amp output.
    let output_pin = gpiod.pd4;

    #[allow(unused)]
    let opa = hal::extend::opa::OpAmp::enable(non_inverting_pin, inverting_pin, output_pin);

    // Pins are available for other uses after disabling opa.
    // let (non_inverting_pin, inverting_pin, output_pin) = opa.disable();

    loop {}
}
