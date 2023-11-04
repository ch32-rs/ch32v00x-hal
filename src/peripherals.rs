use crate::pac;

// We need to export this in the hal for the drivers to use

crate::peripherals! {
    SYSTICK <= SYSTICK,
    USART1 <= USART1,
    SPI1 <= SPI1,
    I2C1 <= I2C1,
    ADC1 <= ADC1,
    EXTI <= EXTI,

    TIM1 <= TIM1,
    TIM2 <= TIM2,

    PD0 <= virtual,
    PD1 <= virtual, // SWIO
    PD2 <= virtual,
    PD3 <= virtual,
    PD4 <= virtual,
    PD5 <= virtual,
    PD6 <= virtual,
    PD7 <= virtual, // nRST

    PA1 <= virtual, // OSCI
    PA2 <= virtual, // OSCO

    PC0 <= virtual,
    PC1 <= virtual,
    PC2 <= virtual,
    PC3 <= virtual,
    PC4 <= virtual,
    PC5 <= virtual,
    PC6 <= virtual,
    PC7 <= virtual,
}
