//! Platform words for 64-bit little-endian targets.

/// Signed machine word (`l4_mword_t`).
#[allow(non_camel_case_types)]
pub type mword = i64;

/// Unsigned machine word (`l4_umword_t`).
#[allow(non_camel_case_types)]
pub type umword = u64;

/// `true` if the platform is big-endian.
pub const BIG_ENDIAN: bool = false;
