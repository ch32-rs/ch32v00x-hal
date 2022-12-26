#![no_std]
#![no_main]

use hal::gpio::GpioExt;
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

    let _clocks = rcc.cfgr.sysclk(8.MHz()).freeze();

    // nanoCH32V203
    let gpioa = peripherals.GPIOA.split();
    let mut led = gpioa.pa15.into_push_pull_output();

    // let gpiob = peripherals.GPIOB;

    // HSI 8MHz
    // 4 opcodes to do a nop sleep here
    let cycle = 8_000_000 / 4;
    loop {
        for _ in 0..cycle {
            unsafe {
                riscv::asm::nop();
            }
        }

        led.toggle();
    }
}
