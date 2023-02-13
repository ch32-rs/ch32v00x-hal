//! HAL for the CH32V20x family of microcontrollers

#![cfg_attr(not(test), no_std)]
#![allow(non_camel_case_types)]

#[cfg(not(feature = "device-selected"))]
compile_error!(
    "This crate requires device feature to be enabled, \
     e.g. `ch32v20x-hal = { version = \"0.1.0\", features = [\"ch32v203g8\"] }`"
);

pub(crate) use embedded_hal as hal;

#[cfg(feature = "ch32v203")]
pub use ch32v2::ch32v20x as pac;

// Enable use of interrupt macro
#[cfg(feature = "rt")]
pub use crate::pac::interrupt;

#[cfg(feature = "device-selected")]
pub mod prelude;

#[cfg(feature = "device-selected")]
pub mod rcc;

#[cfg(feature = "device-selected")]
pub mod gpio;

#[cfg(feature = "device-selected")]
pub mod serial;

#[cfg(feature = "device-selected")]
pub mod signature;

// #[cfg(feature = "device-selected")]
// pub mod timer;

pub mod state {
    /// Indicates that a peripheral is enabled
    pub struct Enabled;

    /// Indicates that a peripheral is disabled
    pub struct Disabled;
}

#[cfg(feature = "device-selected")]
mod sealed {
    pub trait Sealed {}
}
#[cfg(feature = "device-selected")]
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
