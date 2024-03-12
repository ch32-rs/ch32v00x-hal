# ch32v00x-hal

This is a WIP implementation of the embedded-hal traits for the CH32V0 family of microcontrollers.

- [ ] CH32V003: Currently only chip in the family

> **UPDATE**
>
> CH641 is also a RISCV32EC core.

## Peripheral status

- [ ] PWR: Power control
- [x] RCC: Reset and Clock Control
- [x] IWDG: Independent Watchdog
- [ ] WWDG: Window Watchdog
- [ ] FPIC: Programmable Fast Interrupt Controller
- [x] GPIO: General Purpose Input/Output
- [ ] AFIO: Alternate Function Input/Output
- [ ] DMA: Direct Memory Access control
- [x] ADC: Analog to Digital Converter
- [x] ADTM: Advanced control Timer (TIM1)
- [x] GPTM: General Purpose Timer (TIM2)
- [x] USART: Universal Synchronous Asynchronous Receiver Transmitter
- [x] I2C: Inter-intergrated Circuit interface
- [ ] SPI: Serial Peripheral Interface
- [x] ESIG: Electronic Signature
- [ ] FLASH: Flash memory and user option bytes
- [ ] EXTEND: Extended configuration
- [ ] DBG: Debug support

## Guide on Rust with riscv32ec

As of 2023-02-28, The RVE with +c extension for LLVM is merged and shipped with Rust nightly.
Plese refer to `.cargo/config.toml` for the target override, and `rv32ec.json` for the target specification.
Remember to use the latest nightly toolchain.
