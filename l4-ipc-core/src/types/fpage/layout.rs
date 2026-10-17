//! A flexpage is one machine word with four fields:
//!
//! | bits         | field                                               |
//! |--------------|-----------------------------------------------------|
//! | `0..4`       | rights, which depend on the kind                    |
//! | `4..6`       | type: special, memory, I/O ports or capabilities    |
//! | `6..12`      | order, the log2 of the size                         |
//! | `12..`       | page: address, port or capability slot              |

use arbitrary_int::{UInt, u2, u4, u6};
use l4_sys::umword;

/// The raw storage of a flexpage field: a machine word.  The narrow fields
/// ([`RightsType`], [`TypeType`], [`OrderType`]) are backed by `u8`,
/// [`PayloadType`] by the machine word, so this is the common numerator
/// they all extract into.
#[cfg(target_pointer_width = "32")]
pub(crate) type Word = u32;
#[cfg(target_pointer_width = "64")]
pub(crate) type Word = u64;

/// The width of the rights field.
pub type RightsType = u4;

/// The width of the type field.
pub type TypeType = u2;

/// The width of the order field.
pub type OrderType = u6;

/// The width of the page field: the rest of the word.
#[cfg(target_pointer_width = "32")]
pub type PayloadType = arbitrary_int::u20;
#[cfg(target_pointer_width = "64")]
pub type PayloadType = arbitrary_int::u52;

/// The bits of `word` from `LSB` upwards, as a machine-word-wide value.
#[cfg(target_pointer_width = "32")]
pub(crate) fn extract_word<const LSB: usize, const WIDTH: usize>(
    word: umword,
) -> UInt<Word, WIDTH> {
    UInt::<Word, WIDTH>::extract_u32(word as Word, LSB)
}

/// The bits of `word` from `LSB` upwards, as a machine-word-wide value.
#[cfg(target_pointer_width = "64")]
pub(crate) fn extract_word<const LSB: usize, const WIDTH: usize>(
    word: umword,
) -> UInt<Word, WIDTH> {
    UInt::<Word, WIDTH>::extract_u64(word as Word, LSB)
}

/// The four-bit rights field, backed by `u8`.
pub(crate) fn extract_rights(word: umword) -> RightsType {
    RightsType::new(extract_word::<0, 4>(word).value() as u8)
}

/// The two-bit type field, backed by `u8`.
pub(crate) fn extract_type(word: umword) -> TypeType {
    TypeType::new(extract_word::<4, 2>(word).value() as u8)
}

/// The six-bit order field, backed by `u8`.
pub(crate) fn extract_order(word: umword) -> OrderType {
    OrderType::new(extract_word::<6, 6>(word).value() as u8)
}

/// The page field, backed by the machine word.
pub(crate) fn extract_payload(word: umword) -> PayloadType {
    extract_word::<{ super::Page::SHIFT }, { super::Page::BITS }>(word)
}

/// `1` bits for the `WIDTH` low bits, i.e. a mask that fits them.
fn width_mask<const WIDTH: usize>() -> umword {
    ((1u128 << WIDTH) - 1) as umword
}

/// `word` with the `WIDTH` bits at `LSB` replaced by the low bits of
/// `value`.
pub(crate) fn insert<const LSB: usize, const WIDTH: usize>(word: umword, value: umword) -> umword {
    let mask = width_mask::<WIDTH>() << LSB;
    (word & !mask) | ((value << LSB) & mask)
}
