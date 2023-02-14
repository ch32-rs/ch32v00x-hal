#![no_std]
#![no_main]

use core::fmt::Write;
use hal::gpio::GpioExt;
use hal::serial::UartTx;
use hal::signature;
// use hal::rcc::HSEClock;
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
        .config
        //.sysclk_72m_via_hsi()
        .sysclk_144m_via_hsi()
        .freeze();

    // nanoCH32V203: 8MHz HSE, blue LED
    //let gpioa = peripherals.GPIOA.split();
    //let mut led = gpioa.pa15.into_push_pull_output();

    // FlappyBoard: No HSE, ice-blue LED
    let gpiob = peripherals.GPIOB.split();
    let mut led = gpiob.pb8.into_push_pull_output();

    // USART1 init
    // let gpioa = peripherals.GPIOA.split();
    // let tx_pin = gpioa.pa9.into_alternate();
    // let mut usart1 = UartTx::new(peripherals.USART1, tx, &clocks, config);

    // use remaped 01
    let tx_pin = gpiob.pb6.into_alternate();
    // 116200 8N1
    let config = hal::serial::Config::default();
    let mut serial = UartTx::new(peripherals.USART1, tx_pin, &clocks, config);

    write!(&mut serial, "\r\nBooting OK!\r\n").unwrap();
    write!(&mut serial, "RCC init result: {:?}\r\n", clocks).unwrap();

    write!(&mut serial, "MCU Info\r\n").unwrap();

    write!(
        &mut serial,
        "Flash size {}Kib\r\n",
        signature::FlashSize::get().kilo_bytes()
    )
    .unwrap();

    write!(
        &mut serial,
        "Chip UID {:x?}\r\n\r\n",
        signature::Uid::get().as_bytes()
    )
    .unwrap();

    // csr dump
    let misa = riscv::register::misa::read().unwrap();
    write!(
        &mut serial,
        "misa:      {:08x} {:?} ",
        misa.bits(),
        misa.mxl(),
    )
    .unwrap();
    for c in 'A'..='Z' {
        if misa.has_extension(c) {
            write!(&mut serial, "{}", c).unwrap();
        }
    }
    write!(
        &mut serial,
        "\r\nmvendorid: {:?}\r\n",
        riscv::register::mvendorid::read()
    )
    .unwrap();

    let marchid = riscv::register::marchid::read().unwrap().bits();
    write!(&mut serial, "marchid:   {:08x} ", marchid).unwrap();
    // Ref: QingKe V4 Manual
    write!(
        &mut serial,
        "{}{}{}-{}{}{}\r\n",
        (((marchid >> 26) & 0x1F) + 64) as u8 as char,
        (((marchid >> 21) & 0x1F) + 64) as u8 as char,
        (((marchid >> 16) & 0x1F) + 64) as u8 as char,
        (((marchid >> 10) & 0x1F) + 64) as u8 as char,
        ((((marchid >> 5) & 0x1F) as u8) + b'0') as char,
        ((marchid & 0x1F) + 64) as u8 as char,
    )
    .unwrap();

    write!(
        &mut serial,
        "mimpid:    {:08x}\r\n",
        riscv::register::mimpid::read().unwrap().bits(),
    )
    .unwrap();
    write!(
        &mut serial,
        "mhartid:   {:08x}\r\n",
        riscv::register::mhartid::read()
    )
    .unwrap();

    let cycle = 144_000_000 / 2;
    loop {
        unsafe { riscv::asm::delay(cycle) }
        led.toggle();

        // tc check
        write!(&mut serial, "Hello, world!\r\n").unwrap();
        loop {}
    }
}
