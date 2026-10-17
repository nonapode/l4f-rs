//! The common interface of the flexpage kinds.
use super::Type;
use super::layout::{OrderType, PayloadType, RightsType, TypeType};
use core::fmt::Debug;
use core::hash::Hash;
use l4_sys::umword;

mod sealed {
    pub trait Sealed {}
}

/// A kind of flexpage.  The kind fixes the value of the type field, the
/// meaning of the rights field and what the page field holds.
pub trait Kind: sealed::Sealed + Copy + PartialEq + Eq + Hash + Debug {
    /// The value of the type field.
    const TYPE: Type;

    /// Why a flexpage of this kind cannot be built or decoded.
    type Error;

    /// The word of the flexpage, as the kernel expects it.
    fn bits(&self) -> umword;

    /// The raw rights field.
    fn rights_bits(&self) -> RightsType;

    /// The raw order field.
    fn order_bits(&self) -> OrderType;

    /// The raw page field.
    fn payload_bits(&self) -> PayloadType;

    /// The flexpage of the raw fields, if they describe one of this kind.
    fn decode(
        rights: RightsType,
        r#type: TypeType,
        order: OrderType,
        payload: PayloadType,
    ) -> Result<Self, Self::Error>;
}

impl sealed::Sealed for super::Special {}
impl sealed::Sealed for super::Memory {}
impl sealed::Sealed for super::Io {}
impl sealed::Sealed for super::Object {}
