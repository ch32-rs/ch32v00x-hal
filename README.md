# ch32v00x-hal

This is a WIP implementation of the embedded-hal traits for the CH32V0 family of microcontrollers.

- [ ] CH32V003: Currently only chip in the family

> **UPDATE**
>
> CH641 is also a RISCV32EC core.

## Peripheral status

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
- [x] USART: Universal Synchronous Asynchronous Receiver Transmitter
- [x] I2C: Inter-intergrated Circuit interface
- [ ] SPI: Serial Peripheral Interface
- [x] ESIG: Electronic Signature
- [ ] FLASH: Flash memory and user option bytes
- [ ] EXTEND: Extended configuration
- [ ] DBG: Debug support

## Guide on Rust with riscv32ec

The ch32-rs team maintains an updated branch for Rust with riscv32ec support.

### Prerequisites

- A working Rust installation (rustup)
- A nightly Cargo in your channels
- Build essentials (gcc, make, ninja, etc.) and git

### Build a Rust compiler with riscv32ec support

Steps:

- Check out <https://github.com/ch32-rs/rust> branch `rv32ec` (This is a fork of rust-lang/rust) with llvm patched
- `git submodule update --init --recursive`
- Refer <https://noxim.xyz/blog/rust-ch32v003/custom-rust/>, add `config.toml`

    ```toml
    # Use defaults for codegen affecting custom builds
    profile = "codegen"

    [llvm]
    # Use our own LLVM build instead of downloading from CI
    download-ci-llvm = false

    [rust]
    # Enable building LLD for our target as well
    lld = true
    ```

- Start build with `python x.py build`

Check the compiled `rustc` with:

```./build/host/stage1/bin/rustc --print target-list | grep riscv32```

To see if `riscv32ec-unknown-non-elf` is in the list.

### Prepare the toolchain

`rustup toolchain link custom-rv32ec ~/rust/build/host/stage1`

### Project toolchain override

- In this project directory, run `rustup override set custom-rv32ec` to set default toolchain to the one we just built.
- run `cargo --version` and ensure the result is nightly Cargo(otherwise, the easiest way to get one is, run `rustup toolchain install nightly`)

## References

- <https://github.com/Noxime/rust/tree/rv32e>
- <https://noxim.xyz/blog/rust-ch32v003/>
