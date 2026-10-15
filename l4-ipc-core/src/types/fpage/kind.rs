use super::{CapRights, MemRights, Type};
use l4_bitfield::{Bits, Decode, Encode};
use l4_sys::umword;

mod sealed {
    pub trait Sealed {}
}

/// A kind of flexpage.  The kind fixes the value of the type field and what
/// the rights of the flexpage are.
pub trait Kind: sealed::Sealed {
    /// The value of the type field.
    const TYPE: Type;

    /// The rights of a flexpage of this kind.
    type Rights: Copy + Encode<umword, 4> + Decode<umword, 4>;
}

/// The marker of the special flexpages: the invalid one and the one that
/// covers everything.
pub enum Special {}

/// The marker of flexpages for memory.
pub enum Memory {}

/// The marker of flexpages for I/O ports.
pub enum Io {}

/// The marker of flexpages for capabilities.
pub enum Obj {}

/// The marker of a flexpage of any kind, e.g. one that has just been
/// received.  See [`Fpage::resolve`](super::Fpage::resolve).
pub enum Any {}

/// The rights of the special flexpages: none.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default, Debug)]
pub struct NoRights;

impl Encode<umword, 4> for NoRights {
    fn encode(&self) -> Bits<umword, 4> {
        Bits::new(0)
    }
}

impl Decode<umword, 4> for NoRights {
    fn decode(_: Bits<umword, 4>) -> Self {
        NoRights
    }
}

impl sealed::Sealed for Special {}
impl sealed::Sealed for Memory {}
impl sealed::Sealed for Io {}
impl sealed::Sealed for Obj {}

impl Kind for Special {
    const TYPE: Type = Type::Special;
    type Rights = NoRights;
}

impl Kind for Memory {
    const TYPE: Type = Type::Memory;
    type Rights = MemRights;
}

/// The rights of an I/O flexpage are always read-write.
impl Kind for Io {
    const TYPE: Type = Type::Io;
    type Rights = MemRights;
}

impl Kind for Obj {
    const TYPE: Type = Type::Obj;
    type Rights = CapRights;
}
