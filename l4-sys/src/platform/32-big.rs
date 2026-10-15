//! Platform words for 32-bit big-endian targets.

/// Signed machine word (`l4_mword_t`).
#[allow(non_camel_case_types)]
pub type mword = i32;

/// Unsigned machine word (`l4_umword_t`).
#[allow(non_camel_case_types)]
pub type umword = u32;

/// `true` if the platform is big-endian.
pub const BIG_ENDIAN: bool = true;
