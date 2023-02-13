#![no_std]
#![no_main]

use hal::gpio::GpioExt;
use panic_halt as _;
use riscv as _;
use riscv_rt::entry;

use ch32v20x_hal as hal;
use hal::pac;
use hal::prelude::*;

#[entry]
fn main() -> ! {
    let peripherals = pac::Peripherals::take().unwrap();
    let rcc = peripherals.RCC.constrain();

    let _clocks = rcc.config.sysclk_144m_via_hsi().freeze();

    // nanoCH32V203: 8MHz HSE, blue LED
    //let gpioa = peripherals.GPIOA.split();
    //let mut led = gpioa.pa15.into_push_pull_output();

    // FlappyBoard: No HSE, ice-blue LED
    let gpiob = peripherals.GPIOB.split();
    let mut led = gpiob.pb8.into_push_pull_output();

    let cycle = 144_000_000 / 2;
    loop {
        unsafe { riscv::asm::delay(cycle) }
        led.toggle();
    }
}
