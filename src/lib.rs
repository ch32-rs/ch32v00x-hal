//! HAL for the CH32V003 microcontroller

#![no_std]
// #![cfg_attr(not(test), no_std)]
#![allow(non_camel_case_types)]

#[cfg(not(feature = "device-selected"))]
compile_error!(
    "This crate requires device feature to be enabled, \
     e.g. `ch32v00x-hal = { version = \"0.1.0\", features = [\"ch32v003\"] }`"
);

pub use self::peripheral::{Peripheral, PeripheralRef};
pub use self::peripherals::Peripherals;

#[cfg(feature = "ch32v003")]
pub use ch32v0::ch32v003 as pac;

#[cfg(feature = "rt")]
use pac::__EXTERNAL_INTERRUPTS as _;

pub mod adc;
pub mod gpio;
pub mod pwr;
pub mod rcc;
//
// pub mod pfic;
pub mod delay;
pub mod extend;
pub mod i2c;
pub mod serial;
pub mod signature;
pub mod time;
pub mod timer;
pub mod watchdog;

mod critical_section;
pub mod debug;
mod peripheral;
pub mod peripherals;
pub mod prelude;

pub mod state {
    /// Indicates that a peripheral is enabled
    pub struct Enabled;

    /// Indicates that a peripheral is disabled
    pub struct Disabled;
}

mod sealed {
    pub trait Sealed {}
}

pub(crate) use sealed::Sealed;

/// Formatter helper
fn stripped_type_name<T>() -> &'static str {
    let s = core::any::type_name::<T>();
    let p = s.split("::");
    p.last().unwrap()
}

/// Bits per second
pub type BitsPerSecond = fugit::HertzU32;

/// Extension trait that adds convenience methods to the `u32` type
pub trait U32Ext {
    /// Wrap in `Bps`
    fn bps(self) -> BitsPerSecond;
}

impl U32Ext for u32 {
    fn bps(self) -> BitsPerSecond {
        BitsPerSecond::from_raw(self)
    }
}
