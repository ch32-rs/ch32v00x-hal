# ch32v00x-hal

This is a WIP implementation of the embedded-hal traits for the CH32V0 family of microcontrollers.

- [ ] CH32V003: Currently only chip in the family

> **UPDATE**
>
> CH461 is also a RISCV32EC core.

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

To get a working Rust compiler with `riscv32ec-unknown-non-elf` target support is not easy.
There's no undated tutorial on the internet. The following is a guide on how to get a working Rust compiler, based on mine(@andelf's) experience.

### Prerequisites

- A working Rust installation (rustup)
- Build essentials (gcc, make, etc.) and git

### Build a Rust compiler with riscv32ec support

- Rust uses its own llvm fork
- The original <https://reviews.llvm.org/D70401> PR is always get rebased and updated, so we can not use it directly. (A joke, it's created in 2019 and still not merged today 2023-11-03)
- Rust's cargo is always get updated, so it can not be used with an older version of rustc, if long enough time passed.

Steps:

- Check out <https://github.com/Noxime/rust/tree/rv32e> branch `rv32e` (This is a fork of rust-lang/rust) with llvm prepared
- `git submodule update --init --recursive`
- Fix the code:
  - In `compiler/rustc_target/src/spec/riscv32ec_unknown_none_elf.rs`, change `data_layout` to `data_layout: "e-m:e-p:32:32-i64:64-n32-S128".into(),`
- Refer <https://noxim.xyz/blog/rust-ch32v003/custom-rust/>, add `config.toml`
- Start build with `python x.py build`

Check the compiled `rustc` with:

```./build/host/stage1/bin/rustc --print target-list | grep riscv32```

To ses if `riscv32ec-unknown-non-elf` is in the list.

If you prefer to start work from a fresh Rust source tree:

- The `rust-lang/rust` commit hash is: 3de7d7fb22a579a3d59ddb1c959d1b3da224aafa
- The patch for `src/llvm-project` is <https://github.com/rust-lang/llvm-project/commit/a85e08ba59491bb53dedcdd178bf5b27756cd36e.patch>
- You can check what's changed in <https://github.com/Noxime/rust/tree/rv32e> by clicking `4 commits ahead` [link](https://github.com/rust-lang/rust/compare/master...Noxime:rust:rv32e).
- `object` crate fork is not necessary, hack it with the following:

    ```diff
    - if features.contains("+e") {
    -     e_flags |= 0x0008;
    - }
    ```

### Prepare the toolchain

First, refer the tut above: `rustup toolchain link custom-rv32e ~/rust/build/host/stage1`

The original tut no longer works. Due to start1 output does not have a `cargo` matching the `rustc` we just built.
You will get something like `..../sysroot/Cargo.toml` not-found error.

I have to use the following hack:

- Use `rustup toolchain install nightly-2023-02-04` to download an old nightly that almost matches the source code
- `cp ~/.rustup/toolchains/nightly-2023-02-04-x86_64-unknown-linux-gnu/bin/cargo ~/.rustup/toolchains/custom-rv32e/bin`
- yes, ugly but works

### Project toolchain override

In this project, run `rustup override set custom-rv32e` to set default toolchain to the one we just built.

## RISC-V E-extension support

As of 1.70, Rust does not support the RV32EC instruction of the QingKe RISC-V2A core. There is a prototype fork of the
Rust compiler available at <https://github.com/Noxime/rust/tree/rv32e>. Please note the branch "rv32e". For more details,
please read the article <https://noxim.xyz/blog/rust-ch32v003/>
