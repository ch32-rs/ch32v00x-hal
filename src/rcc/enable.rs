use super::*;

macro_rules! bus_enable {
    ($PER:ident => $en:ident) => {
        impl Enable for crate::pac::$PER {
            #[inline(always)]
            fn enable(bus: &mut Self::Bus) {
                bus.enr().modify(|_, w| w.$en().set_bit());
            }
            #[inline(always)]
            fn disable(bus: &mut Self::Bus) {
                bus.enr().modify(|_, w| w.$en().clear_bit());
            }
            #[inline(always)]
            fn is_enabled() -> bool {
                Self::Bus::new().enr().read().$en().bit_is_set()
            }
            #[inline(always)]
            fn is_disabled() -> bool {
                Self::Bus::new().enr().read().$en().bit_is_clear()
            }
            #[inline(always)]
            unsafe fn enable_unchecked() {
                Self::enable(&mut Self::Bus::new());
            }
            #[inline(always)]
            unsafe fn disable_unchecked() {
                Self::disable(&mut Self::Bus::new());
            }
        }
    };
}

macro_rules! bus_reset {
    ($PER:ident => $rst:ident) => {
        impl Reset for crate::pac::$PER {
            #[inline(always)]
            fn reset(bus: &mut Self::Bus) {
                bus.rstr().modify(|_, w| w.$rst().set_bit());
                bus.rstr().modify(|_, w| w.$rst().clear_bit());
            }
            #[inline(always)]
            unsafe fn reset_unchecked() {
                Self::reset(&mut Self::Bus::new());
            }
        }
    };
}

macro_rules! bus {
    ($($PER:ident => ($busX:ty, $($en:ident)?, $($rst:ident)?),)+) => {
        $(
            impl crate::Sealed for crate::pac::$PER {}
            impl RccBus for crate::pac::$PER {
                type Bus = $busX;
            }
            $(bus_enable!($PER => $en);)?
            $(bus_reset!($PER => $rst);)?
        )+
    };
}

bus! {
    // CRC => (AHB, crcen, crcrst)
    // DMA1 => (AHB, dma1en, dma1rst),
    // DMA2 => (AHB, dma2en, dma2rst),

    AFIO => (APB2, afioen, afiorst), // 0
    GPIOA => (APB2, iopaen, ioparst),
    GPIOB => (APB2, iopben, iopbrst),
    GPIOC => (APB2, iopcen, iopcrst),
    GPIOD => (APB2, iopden, iopdrst),
    GPIOE => (APB2, iopeen, ioperst),
    ADC1 => (APB2, adc1en, adc1rst),
    ADC2 => (APB2, adc2en, adc2rst),
    TIM1 => (APB2, tim1en, tim1rst),
    SPI1 => (APB2, spi1en, spi1rst),
    USART1 => (APB2, usart1en, usart1rst),

    TIM2 => (APB1, tim2en, tim2rst), // 0
    TIM3 => (APB1, tim3en, tim3rst),
    TIM4 => (APB1, tim4en, tim4rst),
    WWDG => (APB1, wwdgen, wwdgrst),
    SPI2 => (APB1, spi2en, spi2rst),
    USART2 => (APB1, usart2en, usart2rst),
    USART3 => (APB1, usart3en, usart3rst),
    I2C1 => (APB1, i2c1en, i2c1rst),
    I2C2 => (APB1, i2c2en, i2c2rst),
    CAN1 => (APB1, can1en, can1rst),

    BKP => (APB1, bkpen, bkprst),
    PWR => (APB1, pwren, pwrrst),

    USB => (APB1, usbden, usbdrst),


}
