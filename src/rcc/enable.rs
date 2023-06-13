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
    // 3.4.6
    // SRAM => (AHB, sramen, sramrst), wheres this gone?
    DMA1 => (AHB, dma1en, ),

    // 3.4.7
    AFIO => (APB2, afioen, afiorst), // 0
    GPIOA => (APB2, iopaen, ioparst),
    GPIOC => (APB2, iopcen, iopcrst),
    GPIOD => (APB2, iopden, iopdrst),
    ADC1 => (APB2, adc1en, adc1rst),
    TIM1 => (APB2, tim1en, tim1rst),
    SPI1 => (APB2, spi1en, spi1rst),
    USART1 => (APB2, usart1en, usart1rst),

    // 3.4.8
    TIM2 => (APB1, tim2en, ), // 0
    WWDG => (APB1, wwdgen, wwdgrst),
    I2C1 => (APB1, i2c1en, i2c1rst),
    PWR => (APB1, pwren, pwrrst),
}
