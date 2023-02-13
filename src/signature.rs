//! Device signature.
// Applies for CH32V103, CH32F2x, CH32V2x and CH32V3x.

macro_rules! define_ptr_type {
    ($name: ident, $ptr: expr) => {
        impl $name {
            fn ptr() -> *const Self {
                $ptr as *const _
            }

            /// Returns a wrapped reference to the value in flash memory
            pub fn get() -> &'static Self {
                unsafe { &*Self::ptr() }
            }
        }
    };
}

/// Size of integrated flash
#[derive(Debug)]
#[repr(C)]
pub struct FlashSize(u16);
define_ptr_type!(FlashSize, 0x1FFFF7E0);

impl FlashSize {
    /// Read flash size in kilobytes
    pub fn kilo_bytes(&self) -> u16 {
        self.0
    }

    /// Read flash size in bytes
    pub fn bytes(&self) -> usize {
        usize::from(self.kilo_bytes()) * 1024
    }
}

/// Uniqure Device ID registers
#[derive(Hash, Debug)]
#[repr(C)]
pub struct Uid([u8; 12]);
define_ptr_type!(Uid, 0x1FFFF7E8);

impl Uid {
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}
