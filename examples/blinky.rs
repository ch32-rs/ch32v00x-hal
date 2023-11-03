#![no_std]
#![no_main]

use panic_halt as _;
use riscv_rt::entry;

use ch32v00x_hal::prelude::*;

#[entry]
fn main() -> ! {
    // To ensure safe access to peripherals, all types are !Copy singletons. The
    // PAC makes us pass these marker types around to access the registers
    let p = ch32v0::ch32v003::Peripherals::take().unwrap();

    let mut rcc = p.RCC.constrain();
    let _clocks = rcc.config.freeze();

    let gpiod = p.GPIOD.split(&mut rcc);

    // let tx = gpiod.pd5.into_alternate();
    let mut led = gpiod.pd6.into_push_pull_output();


    loop {
        led.toggle();

        unsafe { riscv::asm::delay(10000000); }
    }
}
