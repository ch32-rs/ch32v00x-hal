#![no_std]
#![no_main]

use panic_halt as _;
use riscv_rt::entry;

use core::fmt::Write;

use ch32v00x_hal::prelude::*;
use ch32v00x_hal::rcc::Clocks;
use ch32v00x_hal::serial::Config;
use ch32v00x_hal::signature::{FlashSize, Uid};

#[entry]
fn main() -> ! {
    // To ensure safe access to peripherals, all types are !Copy singletons. The
    // PAC makes us pass these marker types around to access the registers
    let p = ch32v0::ch32v003::Peripherals::take().unwrap();

    let mut rcc = p.RCC.constrain();

    let clocks = Clocks::default();

    let gpiod = p.GPIOD.split(&mut rcc);

    let tx = gpiod.pd5.into_alternate();
    let rx = gpiod.pd6.into_floating_input();

    let mut usart_config = Config::default();

    // for some reason baudrates above 38400 don't seem possible
    // maybe we need to use the pll clock source for that
    usart_config.baudrate = 38400;

    let mut usart = p.USART1.usart(tx, rx, usart_config, &mut rcc, &clocks);

    let flash_size = FlashSize::get().kilo_bytes();

    let uid_slice = Uid::get().as_bytes();

    let mut uid: u128 = 0;

    for (e, id) in uid_slice.iter().enumerate() {
        uid += (*id as u128) << (12 - e) * 8;
    }

    writeln!(usart, "fs: {:?}\r", flash_size).ok();
    writeln!(usart, "uid: {:?}\r", uid).ok();

    loop {
        let recv: u8 = match nb::block!(usart.read()) {
            Err(_) => continue,
            Ok(recv) => recv,
        };
        nb::block!(usart.write(recv)).unwrap();
        nb::block!(usart.flush()).unwrap();
    }
}
