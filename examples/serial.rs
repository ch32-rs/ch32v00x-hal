#![no_std]
#![no_main]

use core::fmt::Write;
use panic_halt as _;

use ch32v00x_hal as hal;

use hal::prelude::*;
use hal::serial::Config;

#[ch32v_rt::entry]
fn main() -> ! {
    // To ensure safe access to peripherals, all types are !Copy singletons. The
    // PAC makes us pass these marker types around to access the registers
    let p = ch32v0::ch32v003::Peripherals::take().unwrap();

    let mut rcc = p.RCC.constrain();
    let clocks = rcc.config.freeze();

    let gpiod = p.GPIOD.split(&mut rcc);

    let tx = gpiod.pd5.into_alternate();
    let rx = gpiod.pd6.into_floating_input();

    let usart_config = Config::default();

    let mut usart = p.USART1.usart(tx, rx, usart_config, &mut rcc, &clocks);

    let flash_size = hal::signature::flash_size_kb();
    let uid = hal::signature::unique_id();

    writeln!(usart, "flash size: {}KiB\r", flash_size).ok();
    writeln!(usart, "uid: {:02x?}\r", uid).ok();

    loop {
        let recv: u8 = match nb::block!(usart.read()) {
            Err(_) => continue,
            Ok(recv) => recv,
        };
        nb::block!(usart.write(recv)).unwrap();
        nb::block!(usart.flush()).unwrap();
    }
}
