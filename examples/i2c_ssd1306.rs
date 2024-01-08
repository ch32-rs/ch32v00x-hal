#![no_std]
#![no_main]

use core::fmt::Write;
use panic_halt as _;

use ch32v0::ch32v003 as pac;
use ch32v00x_hal as hal;

use hal::{i2c::*, prelude::*};
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};

#[qingke_rt::entry]
fn main() -> ! {
    // Initialize peripherals
    let p = pac::Peripherals::take().unwrap();

    // Configure clocks
    let mut rcc = p.RCC.constrain();
    let clocks = rcc.config.freeze();

    // enable GPIO power domains
    let c = p.GPIOC.split(&mut rcc);

    // I2C pins
    let sda = c.pc1.into_alternate_open_drain();
    let scl = c.pc2.into_alternate_open_drain();

    // Initialize i2c peripheral
    let i2c = I2c::i2c1(p.I2C1, scl, sda, I2cConfig::fast_mode(), &mut rcc, &clocks);

    // Initialize display
    let i2c = I2CDisplayInterface::new(i2c);
    let mut display =
        Ssd1306::new(i2c, DisplaySize128x64, DisplayRotation::Rotate0).into_terminal_mode();

    display.init().unwrap();
    display.clear().unwrap();

    for i in 0.. {
        writeln!(display, "{i}").unwrap();
    }

    loop {}
}
