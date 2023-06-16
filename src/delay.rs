//! Simple busy-loop delay provider

use fugit::HertzU32;

use crate::rcc::Clocks;

pub struct CycleDelay {
    rate: HertzU32,
}

impl CycleDelay {
    pub fn new(clocks: &Clocks) -> CycleDelay { 
        CycleDelay { 
            rate: clocks.hclk()
        }
    }
}

impl embedded_hal_alpha::delay::DelayUs for CycleDelay {
    fn delay_us(&mut self, us: u32) {
        // Widen to u64 to ensure no overflow
        // The QingKe RISC-V2A executes an addi in 2 cycles
        let cycles = us as u64 * self.rate.to_Hz() as u64 / 2_000_000;

        unsafe {
            riscv::asm::delay(cycles as u32);
        }
    }
}

impl embedded_hal::blocking::delay::DelayUs<u32> for CycleDelay {
    fn delay_us(&mut self, us: u32) {
        embedded_hal_alpha::delay::DelayUs::delay_us(self, us as _);
    }
}

impl embedded_hal::blocking::delay::DelayUs<u16> for CycleDelay {
    fn delay_us(&mut self, us: u16) {
        embedded_hal_alpha::delay::DelayUs::delay_us(self, us as _);
    }
}

impl embedded_hal::blocking::delay::DelayUs<u8> for CycleDelay {
    fn delay_us(&mut self, us: u8) {
        embedded_hal_alpha::delay::DelayUs::delay_us(self, us as _);
    }
}
