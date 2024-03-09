INCLUDE memory.x
/* Provides weak aliases (cf. PROVIDED) for device specific interrupt handlers */
/* This will usually be provided by a device crate generated using svd2rust (see `device.x`) */
INCLUDE device.x

PROVIDE(_stext = ORIGIN(REGION_TEXT));
PROVIDE(_stack_start = ORIGIN(REGION_STACK) + LENGTH(REGION_STACK));
PROVIDE(_max_hart_id = 0);
PROVIDE(_hart_stack_size = 2K);
PROVIDE(_heap_size = 0);

/* fault handlers */
PROVIDE(InstructionMisaligned = ExceptionHandler);
PROVIDE(InstructionFault = ExceptionHandler);
PROVIDE(IllegalInstruction = ExceptionHandler);
PROVIDE(Breakpoint = ExceptionHandler);
PROVIDE(LoadMisaligned = ExceptionHandler);
PROVIDE(LoadFault = ExceptionHandler);
PROVIDE(StoreMisaligned = ExceptionHandler);
PROVIDE(StoreFault = ExceptionHandler);;
PROVIDE(UserEnvCall = ExceptionHandler);
PROVIDE(MachineEnvCall = ExceptionHandler);

/* core interrupt handlers */
PROVIDE(NonMaskableInt = DefaultHandler);
PROVIDE(SysTick = DefaultHandler);
PROVIDE(Software = DefaultHandler);

PROVIDE(DefaultHandler = DefaultInterruptHandler);
PROVIDE(ExceptionHandler = DefaultExceptionHandler);

/* # Interrupt vectors */
EXTERN(__EXTERNAL_INTERRUPTS); /* `static` variable similar to `__EXCEPTIONS` */

ENTRY(_start)

SECTIONS
{
    .init :
    {
        . = ALIGN(4);
        KEEP(*(SORT_NONE(.init)))
        . = ALIGN(4);
    } >FLASH AT>FLASH

    .trap :
    {
        . = ALIGN(4);
        KEEP(*(.vector_table.exceptions));
        KEEP(*(.vector_table.interrupts));
        *(.trap.rust)
    } >FLASH AT>FLASH

    .rodata : ALIGN(4)
    {
        *(.srodata .srodata.*);
        *(.rodata .rodata.*);
        . = ALIGN(4);
    } >FLASH AT>FLASH

    .trap1kb  : ALIGN(1024)
    {
        *(.trap)
    } >FLASH AT>FLASH

    .text :
    {
        . = ALIGN(4);
        KEEP(*(SORT_NONE(.handle_reset)))
        *(.init.rust)
        *(.text .text.*)
    } >FLASH AT>FLASH

    .data : ALIGN(4)
    {
        _data_lma = LOADADDR(.data);
        PROVIDE(_data_vma = .);
        PROVIDE( __global_pointer$ = . + 0x800 );
        *(.sdata .sdata.* .sdata2 .sdata2.*);
        *(.data .data.*);
        . = ALIGN(4);
        PROVIDE( _edata = .);
    } >RAM AT>FLASH

    .bss : ALIGN(4)
    {
        PROVIDE( _sbss = .);
        *(.sbss .sbss.* .bss .bss.*);
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
