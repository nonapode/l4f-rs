//! The type field and the page field of a flexpage.
use super::layout::{PayloadType, TypeType};
use arbitrary_int::u2;
use l4_sys::umword;

/// The kind of a flexpage, the two bit type field.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Type {
    /// The special flexpages, see [`Special`](super::Special).
    Special = 0,
    /// A range of memory (`L4_FPAGE_MEMORY`).
    Memory = 1,
    /// A range of I/O ports (`L4_FPAGE_IO`).
    Io = 2,
    /// A range of capability slots (`L4_FPAGE_OBJ`).
    Obj = 3,
}

impl Type {
    /// The raw type field.
    pub const fn to_bits(self) -> TypeType {
        match self {
            Type::Special => u2::new(0),
            Type::Memory => u2::new(1),
            Type::Io => u2::new(2),
            Type::Obj => u2::new(3),
        }
    }

    /// The kind of the raw type field; all four values are valid.
    pub const fn from_bits(bits: TypeType) -> Self {
        match bits.value() {
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
pub struct Page(PayloadType);

impl Page {
    /// The position of the page field within a flexpage
    /// (`L4_FPAGE_ADDR_SHIFT`).
    pub const SHIFT: usize = 12;

    /// The width of the page field (`L4_FPAGE_ADDR_BITS`).
    pub const BITS: usize = umword::BITS as usize - Self::SHIFT;

    /// The page number `page`; panics if it does not fit into the page
    /// field.
    pub fn new(page: umword) -> Page {
        #[cfg(target_pointer_width = "64")]
        let value = PayloadType::new(page as u64);
        #[cfg(target_pointer_width = "32")]
        let value = PayloadType::new(page as u32);
        Page(value)
    }

    /// The page number: the address, port or slot index divided by the
    /// page size.
    pub const fn get(self) -> umword {
        self.0.value() as umword
    }

    /// The page of the raw page field.
    pub(crate) const fn from_payload(payload: PayloadType) -> Page {
        Page(payload)
    }

    /// The raw page field.
    pub(crate) const fn payload(self) -> PayloadType {
        self.0
    }
}

#[cfg(all(test, not(target_os = "none")))]
mod tests {
    use super::*;
    use arbitrary_int::u2;

    #[test]
    fn type_values_match_the_abi() {
        assert_eq!(Type::Special as u8, 0);
        assert_eq!(Type::Memory as u8, 1);
        assert_eq!(Type::Io as u8, 2);
        assert_eq!(Type::Obj as u8, 3);
    }

    #[test]
    fn type_bits_round_trip() {
        for ty in [Type::Special, Type::Memory, Type::Io, Type::Obj] {
            assert_eq!(Type::from_bits(ty.to_bits()), ty);
        }
        assert_eq!(Type::from_bits(u2::new(0)), Type::Special);
        assert_eq!(Type::from_bits(u2::new(3)), Type::Obj);
    }

    #[test]
    fn page_field_is_the_rest_of_the_word() {
        assert_eq!(Page::SHIFT + Page::BITS, umword::BITS as usize);
        assert_eq!(Page::new(0x1000).get(), 0x1000);
    }
}
