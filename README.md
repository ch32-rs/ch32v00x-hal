# ch32v00x-hal

This is a WIP implementation of the embedded-hal traits for the CH32V0 family of microcontrollers.

- [ ] CH32V003: Currently only chip in the family

# Peripheral status

- [ ] PWR: Power control
- [x] RCC: Reset and Clock Control
- [ ] IWDG: Independent Watchdog
- [ ] WWDG: Window Watchdog
- [ ] FPIC: Programmable Fast Interrupt Controller
- [x] GPIO: General Purpose Input/Output
- [ ] AFIO: Alternate Function Input/Output
- [ ] DMA: Direct Memory Access control
- [ ] ADC: Analog to Digital Converter
- [ ] ADTM: Advanced control Timer (TIM1)
- [ ] GPTM: General Purpose Timer (TIM2)
- [ ] USART: Universal Synchronous Asynchronous Receiver Transmitter
- [ ] I2C: Inter-intergrated Circuit interface
- [ ] SPI: Serial Peripheral Interface
- [ ] ESIG: Electronic Signature
- [ ] FLASH: Flash memory and user option bytes
- [ ] EXTEND: Extended configuration
- [ ] DBG: Debug support

# RISC-V E-extension support

As of 1.70, Rust does not support the RV32EC instruction of the QingKe RISC-V2A core. There is a prototype fork of the
Rust compiler available at https://github.com/Noxime/rust/tree/rv32e. Please note the branch "rv32e". For more details,
please read the article https://noxim.xyz/blog/rust-ch32v003/
