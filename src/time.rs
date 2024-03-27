//! Time units
//!
//! See [`Hertz`], [`KiloHertz`] and [`MegaHertz`] for creating increasingly higher frequencies.
//!
//! The [`fugit::ExtU32`] [`U32Ext`] trait adds various methods like `.Hz()`, `.MHz()`, etc to the `u32` primitive type,
//! allowing it to be converted into frequencies.
//!
//! # Examples
//!
//! ## Create a 2 MHz frequency
//!
//! This example demonstrates various ways of creating a 2 MHz (2_000_000 Hz) frequency. They are
//! all equivalent, however the `2.MHz()` variant should be preferred for readability.
//!
//! ```rust
//! use hal::{
//!     time::Hertz,
//!     // Imports U32Ext trait
//!     prelude::*,
//! };
//!
//! let freq_hz = 2_000_000.Hz();
//! let freq_khz = 2_000.kHz();
//! let freq_mhz = 2.MHz();
//!
//! assert_eq!(freq_hz, freq_khz);
//! assert_eq!(freq_khz, freq_mhz);
//! ```

#![allow(non_snake_case)]

use core::ops;

/// Bits per second
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Debug)]
pub struct Bps(pub u32);

pub use fugit::{
    HertzU32 as Hertz, KilohertzU32 as KiloHertz, MegahertzU32 as MegaHertz,
    MicrosDurationU32 as MicroSeconds, MillisDurationU32 as MilliSeconds,
};

/// Extension trait that adds convenience methods to the `u32` type
pub trait U32Ext {
    /// Wrap in `Bps`
    fn bps(self) -> Bps;
}

impl U32Ext for u32 {
    fn bps(self) -> Bps {
        Bps(self)
    }
}

pub const fn Hz(val: u32) -> Hertz {
    Hertz::from_raw(val)
}

pub const fn kHz(val: u32) -> KiloHertz {
    KiloHertz::from_raw(val)
}

pub const fn MHz(val: u32) -> MegaHertz {
    MegaHertz::from_raw(val)
}

pub const fn ms(val: u32) -> MilliSeconds {
    MilliSeconds::from_ticks(val)
}

pub const fn us(val: u32) -> MicroSeconds {
    MicroSeconds::from_ticks(val)
}

/// Macro to implement arithmetic operations (e.g. multiplication, division)
/// for wrapper types.
macro_rules! impl_arithmetic {
    ($wrapper:ty, $wrapped:ty) => {
        impl ops::Mul<$wrapped> for $wrapper {
            type Output = Self;
            fn mul(self, rhs: $wrapped) -> Self {
                Self(self.0 * rhs)
            }
        }

        impl ops::MulAssign<$wrapped> for $wrapper {
            fn mul_assign(&mut self, rhs: $wrapped) {
                self.0 *= rhs;
            }
        }

        impl ops::Div<$wrapped> for $wrapper {
            type Output = Self;
            fn div(self, rhs: $wrapped) -> Self {
                Self(self.0 / rhs)
            }
        }

        impl ops::Div<$wrapper> for $wrapper {
            type Output = $wrapped;
            fn div(self, rhs: $wrapper) -> $wrapped {
                self.0 / rhs.0
            }
        }

        impl ops::DivAssign<$wrapped> for $wrapper {
            fn div_assign(&mut self, rhs: $wrapped) {
                self.0 /= rhs;
            }
        }
    };
}

impl_arithmetic!(Bps, u32);
