#![no_std]
#![no_main]

use panic_halt as _;
use riscv_rt::entry;
use ch32v00x_hal::{gpio::GpioExt, prelude::*, timer::delay::SysDelay};
use embedded_hal::digital::v2::ToggleableOutputPin;
use embedded_hal::blocking::delay::DelayMs;

#[entry]
fn main() -> ! {
    // To ensure safe access to peripherals, all types are !Copy singletons. The
    // PAC makes us pass these marker types around to access the registers
    let p = ch32v0::ch32v003::Peripherals::take().unwrap();

    let mut rcc = p.RCC.constrain();
    let gpioc = p.GPIOC.split(&mut rcc);

    let clocks = rcc.config.freeze();
    let mut delay = SysDelay::new(p.SYSTICK, &clocks);

    let mut pc1 = gpioc.pc1.into_push_pull_output();

    loop {
        // Toggle pin 1
        pc1.toggle();
        delay.delay_ms(500_u16);
    }
}
