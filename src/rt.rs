//! rt for CH58x

use core::arch::global_asm;

#[export_name = "error: riscv-rt appears more than once in the dependency graph"]
#[doc(hidden)]
pub static __ONCE__: () = ();

#[doc(hidden)]
pub union Vector {
    handler: unsafe extern "C" fn(),
    reserved: usize,
}

#[doc(hidden)]
#[no_mangle]
#[allow(unused_variables, non_snake_case)]
pub fn DefaultInterruptHandler() {
    loop {
        // Prevent this from turning into a UDF instruction
        // see rust-lang/rust#28728 for details
        continue;
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u16)]
#[allow(non_camel_case_types)]
pub enum Interrupt {
    NonMaskableInt = 2,
    HardFault = 3,
    // No EcallM, EcallU, BreakPoint
    SysTick = 12,
    Software = 14,

    ///16 - Window Watchdog interrupt
    WWDG = 16,
    ///17 - PVD through EXTI line detection interrupt
    PVD = 17,
    ///18 - Flash global interrupt
    FLASH = 18,
    ///19 - RCC global interrupt
    RCC = 19,
    ///20 - EXTI Line\[7:0\]
    ///interrupt
    EXTI7_0 = 20,
    ///21 - AWU global interrupt
    AWU = 21,
    ///22 - DMA1 Channel1 global interrupt
    DMA1_CHANNEL1 = 22,
    ///23 - DMA1 Channel2 global interrupt
    DMA1_CHANNEL2 = 23,
    ///24 - DMA1 Channel3 global interrupt
    DMA1_CHANNEL3 = 24,
    ///25 - DMA1 Channel4 global interrupt
    DMA1_CHANNEL4 = 25,
    ///26 - DMA1 Channel5 global interrupt
    DMA1_CHANNEL5 = 26,
    ///27 - DMA1 Channel6 global interrupt
    DMA1_CHANNEL6 = 27,
    ///28 - DMA1 Channel7 global interrupt
    DMA1_CHANNEL7 = 28,
    ///29 - ADC global interrupt
    ADC = 29,
    ///30 - I2C1 event interrupt
    I2C1_EV = 30,
    ///31 - I2C1 error interrupt
    I2C1_ER = 31,
    ///32 - USART1 global interrupt
    USART1 = 32,
    ///33 - SPI1 global interrupt
    SPI1 = 33,
    ///34 - TIM1 Break interrupt
    TIM1_BRK = 34,
    ///35 - TIM1 Update interrupt
    TIM1_UP = 35,
    ///36 - TIM1 Trigger and Commutation interrupts
    TIM1_TRG_COM = 36,
    ///37 - TIM1 Capture Compare interrupt
    TIM1_CC = 37,
    ///38 - TIM2 global interrupt
    TIM2 = 38,
}

// Overwrites PAC's interrupt handlers
extern "C" {
    fn NonMaskableInt();

    fn HardFault();

    fn SysTick();

    fn Software();

    // External interrupts
    fn WWDG();
    fn PVD();
    fn FLASH();
    fn RCC();
    fn EXTI7_0();
    fn AWU();
    fn DMA1_CHANNEL1();
    fn DMA1_CHANNEL2();
    fn DMA1_CHANNEL3();
    fn DMA1_CHANNEL4();
    fn DMA1_CHANNEL5();
    fn DMA1_CHANNEL6();
    fn DMA1_CHANNEL7();
    fn ADC();
    fn I2C1_EV();
    fn I2C1_ER();
    fn USART1();
    fn SPI1();
    fn TIM1_BRK();
    fn TIM1_UP();
    fn TIM1_TRG_COM();
    fn TIM1_CC();
    fn TIM2();
}

#[doc(hidden)]
#[link_section = ".vector_table.interrupts"]
#[no_mangle]
pub static __INTERRUPTS: [Vector; 38] = [
    // Vector { reserved: 0 }, // 0, jump instruction
    Vector { reserved: 0 }, // reset?
    // 2: Non-Maskable Interrupt.
    Vector {
        handler: NonMaskableInt,
    },
    // 3: Hard Fault Interrupt.
    Vector { handler: HardFault },
    Vector { reserved: 0 },
    Vector { reserved: 0 },
    Vector { reserved: 0 },
    Vector { reserved: 0 },
    Vector { reserved: 0 },
    Vector { reserved: 0 },
    // 10-11
    Vector { reserved: 0 },
    Vector { reserved: 0 },
    // 12
    Vector { handler: SysTick },
    Vector { reserved: 0 },
    Vector { handler: Software },
    Vector { reserved: 0 },
    // External interrupts
    Vector { handler: WWDG },
    Vector { handler: PVD },
    Vector { handler: FLASH },
    Vector { handler: RCC },
    Vector { handler: EXTI7_0 },
    Vector { handler: AWU },
    Vector {
        handler: DMA1_CHANNEL1,
    },
    Vector {
        handler: DMA1_CHANNEL2,
    },
    Vector {
        handler: DMA1_CHANNEL3,
    },
    Vector {
        handler: DMA1_CHANNEL4,
    },
    Vector {
        handler: DMA1_CHANNEL5,
    },
    Vector {
        handler: DMA1_CHANNEL6,
    },
    Vector {
        handler: DMA1_CHANNEL7,
    },
    Vector { handler: ADC },
    Vector { handler: I2C1_EV },
    Vector { handler: I2C1_ER },
    Vector { handler: USART1 },
    Vector { handler: SPI1 },
    Vector { handler: TIM1_BRK },
    Vector { handler: TIM1_UP },
    Vector {
        handler: TIM1_TRG_COM,
    },
    Vector { handler: TIM1_CC },
    Vector { handler: TIM2 },
];

macro_rules! cfg_global_asm {
    {@inner, [$($x:tt)*], } => {
        global_asm!{$($x)*}
    };
    (@inner, [$($x:tt)*], #[cfg($meta:meta)] $asm:literal, $($rest:tt)*) => {
        #[cfg($meta)]
        cfg_global_asm!{@inner, [$($x)* $asm,], $($rest)*}
        #[cfg(not($meta))]
        cfg_global_asm!{@inner, [$($x)*], $($rest)*}
    };
    {@inner, [$($x:tt)*], $asm:literal, $($rest:tt)*} => {
        cfg_global_asm!{@inner, [$($x)* $asm,], $($rest)*}
    };
    {$($asms:tt)*} => {
        cfg_global_asm!{@inner, [], $($asms)*}
    };
}

cfg_global_asm! {
    "
    .section    .init,\"ax\"
    .global _start
    .align  1
//    .option norvc
_start:
    j handle_reset
    ",
    "
    .section    .handle_reset,\"ax\",@progbits
    .weak   handle_reset
    .align  1
handle_reset:
    .option push
    .option norelax
    la gp, __global_pointer$
    .option pop
    la sp, _stack_top
    ",
    // load data from flash to ram
    "
2:
    la a0, _data_lma
    la a1, _data_vma
    la a2, _edata
    bgeu a1, a2, 2f
1:
    lw t0, (a0)
    sw t0, (a1)
    addi a0, a0, 4
    addi a1, a1, 4
    bltu a1, a2, 1b
2:
    ",
    // clear bss section
    "
    la a0, _sbss
    la a1, _ebss
    bgeu a0, a1, 2f
1:
    sw zero, (a0)
    addi a0, a0, 4
    bltu a0, a1, 1b
2:
    ",
    // 打开嵌套中断、硬件压栈功能
    // intsyscr: Open nested interrupts and hardware stack functions
    // 0x3 both nested interrupts and hardware stack
    // 0x1 only hardware stack
    "
    li t0, 0x3
    csrw 0x804, t0",
    // Restore state, QingKe V2 has machine mode only
    "
    li t0, 0x80
    csrs mstatus, t0
    ",
    // 配置向量表模式为绝对地址模式. 1KB aligned, use zero address
    "
    la t0, _start
    ori t0, t0, 3
    csrw mtvec, t0
    ",
    "
    la t0, main
    csrw mepc, t0

    mret
    ",
}
