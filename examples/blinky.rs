#![no_std]
#![no_main]

use hal::gpio::GpioExt;
use hal::rcc::HSEClock;
use riscv_rt::entry;
// provide implementation for critical-section
use panic_halt as _;
use riscv as _;

use ch32v20x_hal as hal;
use hal::pac;
use hal::prelude::*;

#[entry]
fn main() -> ! {
    let peripherals = pac::Peripherals::take().unwrap();
    let rcc = peripherals.RCC.constrain();

    let clocks = rcc
        .cfgr
        .sysclk(48.MHz())
        // Or use HSE
        // .hse(HSEClock::new(8.MHz(), hal::rcc::HSEClockMode::Oscillator))
        .freeze();

    // nanoCH32V203: 8MHz HSE, blue LED
    //let gpioa = peripherals.GPIOA.split();
    //let mut led = gpioa.pa15.into_push_pull_output();

    // FlappyBoard: No HSE, ice-blue LED
    let gpiob = peripherals.GPIOB.split();
    let mut led = gpiob.pb8.into_push_pull_output();

    // HSI 8MHz
    let cycle = 8_000_000 / 10;
    loop {
        unsafe { riscv::asm::delay(cycle) }
        led.toggle();
    }
}
