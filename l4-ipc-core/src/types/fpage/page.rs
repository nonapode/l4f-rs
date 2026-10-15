use l4_bitfield::{Bits, Decode, Encode};
use l4_sys::umword;

/// The kind of a flexpage, the two bit type field.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Type {
    /// The special flexpages, see [`Fpage::invalid`](super::Fpage::invalid)
    /// and [`Fpage::all`](super::Fpage::all).
    Special = 0,
    /// A range of memory (`L4_FPAGE_MEMORY`).
    Memory = 1,
    /// A range of I/O ports (`L4_FPAGE_IO`).
    Io = 2,
    /// A range of capability slots (`L4_FPAGE_OBJ`).
    Obj = 3,
}

impl Encode<umword, 2> for Type {
    fn encode(&self) -> Bits<umword, 2> {
        Bits::new(*self as umword)
    }
}

impl Decode<umword, 2> for Type {
    fn decode(bits: Bits<umword, 2>) -> Self {
        // All four values of the field are valid.
        match bits.into_inner() {
            0 => Type::Special,
            1 => Type::Memory,
            2 => Type::Io,
            _ => Type::Obj,
        }
    }
}

/// The page field of a flexpage, the upper bits above the page shift.
///
/// What the page number is depends on the kind of flexpage: for memory it
/// is the address divided by the page size, for capabilities the index of
/// the capability slot and for I/O ports the port number.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Page(umword);

impl Page {
    /// The position of the page field within a flexpage
    /// (`L4_FPAGE_ADDR_SHIFT`).
    pub const SHIFT: usize = 12;

    /// The width of the page field (`L4_FPAGE_ADDR_BITS`).
    pub const BITS: usize = umword::BITS as usize - Self::SHIFT;

    pub const fn new(page: umword) -> Page {
        Page(page)
    }

    pub const fn get(self) -> umword {
        self.0
    }
}

impl<const N: usize> Encode<umword, N> for Page {
    fn encode(&self) -> Bits<umword, N> {
        Bits::new(self.0)
    }
}

impl<const N: usize> Decode<umword, N> for Page {
    fn decode(bits: Bits<umword, N>) -> Self {
        Page(bits.into_inner())
    }
}

#[cfg(all(test, not(target_os = "none")))]
mod tests {
    use super::*;

    #[test]
    fn type_values_match_the_abi() {
        assert_eq!(Type::Special as u8, 0);
        assert_eq!(Type::Memory as u8, 1);
        assert_eq!(Type::Io as u8, 2);
        assert_eq!(Type::Obj as u8, 3);
    }

    #[test]
    fn type_codec_round_trips() {
        for ty in [Type::Special, Type::Memory, Type::Io, Type::Obj] {
            assert_eq!(Type::decode(ty.encode()), ty);
        }
    }

    #[test]
    fn page_field_is_the_rest_of_the_word() {
        assert_eq!(Page::SHIFT + Page::BITS, umword::BITS as usize);
    }
}
