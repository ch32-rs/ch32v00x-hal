# ch32v20x-hal

This is a WIP implementation of the embedded-hal traits for the CH32V2 family of microcontrollers.

- [ ] CH32V203: QingKe V4B, Small-and-medium capacity general-purpose device
- [ ] CH32V208: QingKe V4C, Wireless device

## How to Run

```shell
cargo build --release --example blinky

wchisp flash ./target/riscv32imac-unknown-none-elf/release/examples/blinky
```

```console
> cargo build --release --example serial

> wchisp flash ./target/riscv32imac-unknown-none-elf/release/examples/serial

> # serial output:
Booting OK!
RCC: Clocks { sysclk: Rate { raw: 144000000 }, hclk: Rate { raw: 144000000 }, pclk1: Rate { raw: 144000000 }, pclk2: Rate { raw: 144000000 }, pllclk: Some(Rate { raw: 144000000 }), hse: None, lse: None, lsi: None }
MCU Info
misa:      40901105 XLEN32 ACIMUX
mvendorid: None
marchid:   dc68d882 WCH-V4B
mimpid:    dc688001
mhartid:   00000000
Hello, world!
```

## NOTES on Part Number

```text
CH32V203G6U6
        ||||
        |||`-> Temperature range
        ||`--> Package: QFN
        |`---> Flash Size
        `----> Pin Count
```

Flash size:

- 4 = 16K
- 6 = 32K
- 8 = 64K
- B = 128K
- C = 256K


- D6 32KB or 64KB, Low-and-medium-density general
- D8 128KB or 256KB, High-density general
- D8C 128KB or 256KB, Connectivity or interconnectivity
- D8W 128KB or 256KB, Wireless

```
CH32V20x_D6: CH32V203F6, CH32V203G6, CH32V203K6, CH32V203F8, CH32V203G8, CH32V203K8, CH32V203C6 and CH32V203C8.
CH32V20x_D8: CH32V203RB.
CH32V20x_D8W: CH32V208GB, CH32V208CB, CH32V208RB and CH32V208WB.

CH32V30x_D8: CH32V303CB, CH32V303RB, CH32V303RC and CH32V303VC.
CH32V30x_D8C: CH32V305FB, CH32V305RB, CH32V307RC, CH32V307WC and CH32V307VC.
```

## NOTES on PLL variants

- CH32F20x_D8C  CH32V30x_D8C
- CH32F20x_D6、CH32F20x_D8、CH32V20x_D6 和 CH32V30x_D8
- CH32V203RB (has ETH-PHY)
- CH32F20x_D8W、CH32V20x_D8, CH32V20x_D8W
