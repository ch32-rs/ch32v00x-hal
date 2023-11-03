//! Device signature.
// Applies for CH32V103, CH32F2x, CH32V2x and CH32V3x.

use core::ptr;

const ESIG_FLACAP: u32 = 0x1FFFF7E0;
const ESIG_UNIID: u32 = 0x1FFFF7E8;

/// Flash size in KiB.
#[inline]
pub fn flash_size_kb() -> u16 {
    unsafe { ptr::read_volatile(ESIG_FLACAP as *const u16) }
}

/// UID
#[inline]
pub fn unique_id() -> &'static [u8; 12] {
    unsafe { &(*(ESIG_UNIID as *const [u8; 12])) }
}
