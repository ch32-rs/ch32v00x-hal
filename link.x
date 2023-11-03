INCLUDE memory.x

PROVIDE(NonMaskableInt = DefaultHandler);
PROVIDE(HardFault = DefaultHandler);
PROVIDE(SysTick = DefaultHandler);
PROVIDE(Software = DefaultHandler);

PROVIDE(WWDG = DefaultHandler);
PROVIDE(PVD = DefaultHandler);
PROVIDE(FLASH = DefaultHandler);
PROVIDE(RCC = DefaultHandler);
PROVIDE(EXTI7_0 = DefaultHandler);
PROVIDE(AWU = DefaultHandler);
PROVIDE(DMA1_CHANNEL1 = DefaultHandler);
PROVIDE(DMA1_CHANNEL2 = DefaultHandler);
PROVIDE(DMA1_CHANNEL3 = DefaultHandler);
PROVIDE(DMA1_CHANNEL4 = DefaultHandler);
PROVIDE(DMA1_CHANNEL5 = DefaultHandler);
PROVIDE(DMA1_CHANNEL6 = DefaultHandler);
PROVIDE(DMA1_CHANNEL7 = DefaultHandler);
PROVIDE(ADC = DefaultHandler);
PROVIDE(I2C1_EV = DefaultHandler);
PROVIDE(I2C1_ER = DefaultHandler);
PROVIDE(USART1 = DefaultHandler);
PROVIDE(SPI1 = DefaultHandler);
PROVIDE(TIM1_BRK = DefaultHandler);
PROVIDE(TIM1_UP = DefaultHandler);
PROVIDE(TIM1_TRG_COM = DefaultHandler);
PROVIDE(TIM1_CC = DefaultHandler);
PROVIDE(TIM2 = DefaultHandler);

PROVIDE(DefaultHandler = DefaultInterruptHandler);

ENTRY(_start)

SECTIONS
{
    .init :
    {
        . = ALIGN(4);
        KEEP(*(SORT_NONE(.init)))
        . = ALIGN(4);
    } >FLASH AT>FLASH

    .vector_table :
    {
        . = ALIGN(4);
        KEEP(*(.vector_table.interrupts));
        . = ALIGN(4);
    } >FLASH AT>FLASH

    .text :
    {
        . = ALIGN(4);
        KEEP(*(SORT_NONE(.handle_reset)))
        *(.text .text.*)
    } >FLASH AT>FLASH

    .rodata : ALIGN(4)
    {
        *(.srodata .srodata.*);
        *(.rodata .rodata.*);
    } >FLASH AT>FLASH

    .data : ALIGN(4)
    {
        _data_lma = LOADADDR(.data);
        PROVIDE(_data_vma = .);
        *(.data .data.*)
        . = ALIGN(8);
        PROVIDE( __global_pointer$ = . + 0x800 );
        /* These sections are used by the BLE lib */
        *(.sdata .sdata.*)
        . = ALIGN(4);
        PROVIDE( _edata = .);
    } >RAM AT>FLASH

    .bss : ALIGN(4)
    {
        PROVIDE( _sbss = .);
        *(.sbss .sbss.*)
        *(.bss .bss.*)
        *(.gnu.linkonce.sb.*)
        *(.gnu.linkonce.b.*)
        *(COMMON*)
        PROVIDE( _ebss = .);
    } >RAM AT>FLASH

    .stack ORIGIN(RAM)+LENGTH(RAM) :
    {
        . = ALIGN(4);
        PROVIDE(_stack_top = . );
    } >RAM

    .got (INFO) :
    {
        KEEP(*(.got .got.*));
    }

    .eh_frame (INFO) : { KEEP(*(.eh_frame)) }
    .eh_frame_hdr (INFO) : { *(.eh_frame_hdr) }
}
