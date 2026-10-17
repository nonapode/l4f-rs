//! Flexpages: a range of memory, I/O ports or capability slots, used to
//! describe what is mapped or granted by an IPC.
//!
//! A flexpage is one machine word with four fields:
//!
//! | bits         | field                                               |
//! |--------------|-----------------------------------------------------|
//! | `0..4`       | rights, which depend on the kind                    |
//! | `4..6`       | type: special, memory, I/O ports or capabilities    |
//! | `6..12`      | order, the log2 of the size                         |
//! | `12..`       | page: address, port or capability slot              |
//!
//! [`Fpage`] is typed with the kind of flexpage, so that the rights of a
//! capability cannot be used for memory:
//!
//! ```
//! use l4_ipc_core::types::fpage::{Fpage, MemRights, Memory, Order};
//!
//! let page = Fpage::<Memory>::new(0x1000, Order::PAGE, MemRights::RW)?;
//! assert_eq!(page.rights(), MemRights::RW);
//! assert_eq!(page.order(), Order::PAGE);
//! assert_eq!(page.addr(), 0x1000);
//! # Ok::<(), l4_ipc_core::types::fpage::memory::Error>(())
//! ```
//!
//! ```compile_fail,E0308
//! use l4_ipc_core::types::fpage::{CapRights, Fpage, Memory, Order};
//!
//! // Capability rights do not mean anything for a memory flexpage.
//! let page = Fpage::<Memory>::new(0x1000, Order::PAGE, CapRights::RW);
//! ```
//!
//! A flexpage that has just been received is [`Generic`]: only the type
//! field says what it is.  The conversion into a kind checks the fields
//! and yields the reason when they do not fit:
//!
//! ```
//! use l4_ipc_core::types::fpage::{Fpage, Generic, MemRights, Memory, Order};
//!
//! let word = Generic::from_bits(0x1316);
//! let page: Fpage<Memory> = word.try_into()?;
//! assert_eq!(page.addr(), 0x1000);
//! # Ok::<(), l4_ipc_core::types::fpage::memory::Error>(())
//! ```

use crate::CapIdx;
use core::fmt;
use core::hash::{Hash, Hasher};
use l4_sys::umword;

mod error;
mod flags;
pub mod io;
mod kind;
pub mod layout;
mod marshal;
pub mod memory;
pub mod object;
mod order;
mod page;
mod rights;
pub mod snd_base;
pub mod special;

pub use error::Error;
pub use io::Io;
pub use kind::Kind;
pub use layout::{OrderType, PayloadType, RightsType, TypeType};
pub use memory::Memory;
pub use object::Object;
pub use order::Order;
pub use page::{Page, Type};
pub use rights::{CapRights, MemRights, NoRights};
pub use snd_base::{Cacheability, ObjOptions, Options, SndBase, Transfer};
pub use special::Special;

const _: () = {
    assert!(Page::SHIFT == 12);
    assert!(umword::BITS == usize::BITS);
    assert!(Page::SHIFT + Page::BITS == umword::BITS as usize);
    assert!(PayloadType::BITS == umword::BITS as usize - Page::SHIFT);
};

/// A flexpage of any kind: a raw word, e.g. one received in a message.
///
/// The kind is only known from the type field, which is checked when the
/// flexpage is converted into a [`Fpage`] of a kind.  `Generic` is `Copy`,
/// so the word is still available after a conversion has failed.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Generic(umword);

impl Generic {
    /// The flexpage for a raw word, e.g. one received in a message.
    pub const fn from_bits(bits: umword) -> Self {
        Generic(bits)
    }

    /// The raw word, as it is stored in a message register.
    pub const fn bits(self) -> umword {
        self.0
    }

    /// The kind according to the type field.
    pub fn kind(self) -> Type {
        Type::from_bits(layout::extract_type(self.0))
    }

    /// The four fields of the word.
    fn parts(self) -> (RightsType, TypeType, OrderType, PayloadType) {
        (
            layout::extract_rights(self.0),
            layout::extract_type(self.0),
            layout::extract_order(self.0),
            layout::extract_payload(self.0),
        )
    }

    /// The flexpage as the kind `K`, if its type field says so.
    pub fn try_into_kind<K: Kind>(self) -> Result<K, K::Error> {
        let (rights, r#type, order, payload) = self.parts();
        K::decode(rights, r#type, order, payload)
    }
}

impl fmt::Debug for Generic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (rights, r#type, order, payload) = self.parts();

        macro_rules! attempt {
            ($kind:ty) => {
                if let Ok(kind) = <$kind as Kind>::decode(rights, r#type, order, payload) {
                    return f.debug_tuple("Generic").field(&kind).finish();
                }
            };
        }
        attempt!(Special);
        attempt!(Memory);
        attempt!(Io);
        attempt!(Object);

        write!(f, "Generic(Invalid({:#x}))", self.0)
    }
}

/// A flexpage (`l4_fpage_t`) of the kind `K`: only valid states can be
/// built.
#[repr(transparent)]
pub struct Fpage<K: Kind>(K);

impl<K: Kind> Clone for Fpage<K> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<K: Kind> Copy for Fpage<K> {}

impl<K: Kind> PartialEq for Fpage<K> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<K: Kind> Eq for Fpage<K> {}

impl<K: Kind> Hash for Fpage<K> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

impl<K: Kind> fmt::Debug for Fpage<K> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Fpage")
            .field(&format_args!("{:#x}", self.bits()))
            .finish()
    }
}

impl<K: Kind> Fpage<K> {
    /// The flexpage of the raw fields, if they describe a flexpage of this
    /// kind.
    pub fn decode(
        rights: RightsType,
        order: Order,
        payload: PayloadType,
    ) -> Result<Self, K::Error> {
        K::decode(rights, K::TYPE.to_bits(), order.bits(), payload).map(Fpage)
    }

    /// The raw word, as it is stored in a message register.
    pub fn bits(self) -> umword {
        self.0.bits()
    }

    /// The flexpage as its kind.
    pub fn into_inner(self) -> K {
        self.0
    }
}

impl<K: Kind> From<Fpage<K>> for Generic {
    fn from(fpage: Fpage<K>) -> Generic {
        Generic(fpage.bits())
    }
}

impl<K: Kind> From<K> for Generic {
    fn from(kind: K) -> Generic {
        Generic(kind.bits())
    }
}

impl<K: Kind> TryFrom<Generic> for Fpage<K> {
    type Error = K::Error;

    fn try_from(any: Generic) -> Result<Self, K::Error> {
        let (rights, r#type, order, payload) = any.parts();
        K::decode(rights, r#type, order, payload).map(Fpage)
    }
}

/// Whether `value` is a multiple of the size `1 << order`.  The order of
/// the whole address space is not a size but means everything, which
/// starts at 0.
fn aligned(value: umword, order: Order) -> bool {
    if order == Order::WHOLE_SPACE {
        return value == 0;
    }
    match (1 as umword).checked_shl(order.get().into()) {
        Some(size) => value & (size - 1) == 0,
        None => value == 0,
    }
}

// The accessors of a kind are forwarded to its flexpage, so that
// `Fpage::<Memory>::new` builds the flexpage directly.
macro_rules! forward {
    ($kind:ident { $($item:tt)* }) => {
        impl Fpage<$kind> {
            $($item)*
        }
    };
}

forward!(Memory {
    /// The memory flexpage of `1 << order` bytes at `addr`
    /// (`l4_fpage`).
    ///
    /// The order cannot be smaller than a page, and the address has to be
    /// a multiple of the size.
    pub fn new(addr: umword, order: Order, rights: MemRights) -> Result<Self, memory::Error> {
        Memory::new(addr, order, rights).map(Fpage)
    }

    /// The address of the first byte.
    pub fn addr(self) -> umword {
        self.0.addr()
    }

    /// The size of the flexpage as a power of two.
    pub fn order(self) -> Order {
        self.0.order()
    }

    /// The rights of the mapping.
    pub fn rights(self) -> MemRights {
        self.0.rights()
    }

    /// The same flexpage with other rights (`l4_fpage_set_rights`).
    #[must_use]
    pub fn with_rights(self, rights: MemRights) -> Self {
        Fpage(self.0.with_rights(rights))
    }
});

forward!(Io {
    /// The I/O flexpage of `1 << order` ports at `port` (`l4_iofpage`).
    /// Its rights are read-write.
    ///
    /// The order is at most that of the whole I/O port space, and the port
    /// has to be a multiple of the size.
    pub fn new(port: u16, order: Order) -> Result<Self, io::Error> {
        Io::new(port, order).map(Fpage)
    }

    /// The first port.
    pub fn port(self) -> u16 {
        self.0.port()
    }

    /// The size of the flexpage as a power of two.
    pub fn order(self) -> Order {
        self.0.order()
    }

    /// The rights of the mapping: I/O ports are always mapped
    /// read-write.
    pub fn rights(self) -> MemRights {
        self.0.rights()
    }
});

forward!(Object {
    /// The capability flexpage of `1 << order` slots starting at `slot`
    /// (`l4_obj_fpage`).
    ///
    /// The selector has to be valid, the order cannot exceed the number of
    /// bits of a slot index, and the slot has to be a multiple of the size.
    pub fn new(slot: CapIdx, order: Order, rights: CapRights) -> Result<Self, object::Error> {
        Object::new(slot, order, rights).map(Fpage)
    }

    /// The selector of the first capability slot.
    pub fn cap(self) -> CapIdx {
        self.0.cap()
    }

    /// The size of the flexpage as a power of two.
    pub fn order(self) -> Order {
        self.0.order()
    }

    /// The rights of the mapped capability.
    pub fn rights(self) -> CapRights {
        self.0.rights()
    }

    /// The same flexpage with other rights (`l4_fpage_set_rights`).
    #[must_use]
    pub fn with_rights(self, rights: CapRights) -> Self {
        Fpage(self.0.with_rights(rights))
    }
});

forward!(Special {
    /// The flexpage that is not valid (`l4_fpage_invalid`).
    pub const fn invalid() -> Self {
        Fpage(Special::invalid())
    }

    /// The flexpage that covers all spaces (`l4_fpage_all`).
    pub const fn all() -> Self {
        Fpage(Special::all())
    }

    /// The size of the range as a power of two.
    pub fn order(self) -> Order {
        self.0.order()
    }

    /// The rights of a special flexpage: none.
    pub fn rights(self) -> NoRights {
        self.0.rights()
    }
});

#[cfg(all(test, not(target_os = "none")))]
mod tests {
    use super::*;
    use arbitrary_int::{u2, u4};

    fn order(order: u8) -> Order {
        Order::new(order).unwrap()
    }

    // The expected words are what the C helpers of l4sys/include/__l4_fpage.h
    // compute: rights | type << 4 | order << 6 | address.

    #[test]
    fn memory_flexpage_matches_l4_fpage() {
        let page = Fpage::<Memory>::new(0x1000, Order::PAGE, MemRights::RW).unwrap();

        assert_eq!(page.bits(), 0x6 | 1 << 4 | 12 << 6 | 0x1000);
        assert_eq!(page.bits(), 0x1316);
        assert_eq!(page.addr(), 0x1000);
        assert_eq!(page.order(), Order::PAGE);
        assert_eq!(page.rights(), MemRights::RW);
    }

    #[test]
    fn io_flexpage_matches_l4_iofpage() {
        let page = Fpage::<Io>::new(0x3f8, order(3)).unwrap();

        assert_eq!(page.bits(), 0x6 | 2 << 4 | 3 << 6 | 0x3f8 << 12);
        assert_eq!(page.bits(), 0x3f_80e6);
        assert_eq!(page.port(), 0x3f8);
        assert_eq!(page.order(), order(3));
        assert_eq!(page.rights(), MemRights::RW);
    }

    #[test]
    fn object_flexpage_matches_l4_obj_fpage() {
        let factory = CapIdx::from_index(2);
        let page = Fpage::<Object>::new(factory, order(0), CapRights::RWS).unwrap();

        assert_eq!(page.bits(), 0x7 | 3 << 4 | factory.bits());
        assert_eq!(page.bits(), 0x2037);
        assert_eq!(page.cap(), factory);
        assert_eq!(page.order(), order(0));
        assert_eq!(page.rights(), CapRights::RWS);
    }

    #[test]
    fn special_flexpages_match_the_c_helpers() {
        assert_eq!(Fpage::<Special>::invalid().bits(), 0);
        assert_eq!(Fpage::<Special>::all().bits(), 63 << 6);
        assert_eq!(Fpage::<Special>::all().order(), Order::WHOLE_SPACE);
        assert_eq!(Fpage::<Special>::all().rights(), NoRights);
    }

    #[test]
    fn words_decode_into_the_same_flexpage() {
        let page = Fpage::<Memory>::new(0x1000, Order::PAGE, MemRights::RW).unwrap();
        let received = Fpage::<Memory>::try_from(Generic::from(page)).unwrap();

        assert_eq!(received, page);
        assert_eq!(received.bits(), page.bits());
    }

    #[test]
    fn with_rights_keeps_everything_else() {
        let page = Fpage::<Memory>::new(0x4000, order(14), MemRights::RWX).unwrap();
        let read_only = page.with_rights(MemRights::R);

        assert_eq!(read_only.bits(), page.bits() & !0xf | 0x4);
        assert_eq!(read_only.addr(), 0x4000);
        assert_eq!(read_only.order(), order(14));
    }

    #[test]
    fn memory_order_and_alignment_are_checked() {
        let new = |addr, order| Fpage::<Memory>::new(addr, order, MemRights::R);

        assert_eq!(new(0, order(11)), Err(memory::Error::Order));
        assert_eq!(new(0x1000, order(13)), Err(memory::Error::Misaligned));
        assert_eq!(new(0x1800, Order::PAGE), Err(memory::Error::Misaligned));
        assert!(new(0x2000, order(13)).is_ok());
    }

    #[test]
    fn the_whole_address_space_starts_at_zero() {
        let new = |addr| Fpage::<Memory>::new(addr, Order::WHOLE_SPACE, MemRights::RWX);

        assert!(new(0).is_ok());
        assert_eq!(new(0x1000), Err(memory::Error::Misaligned));
        assert_eq!(new(1 << 63), Err(memory::Error::Misaligned));
    }

    #[test]
    fn io_order_and_alignment_are_checked() {
        assert_eq!(Fpage::<Io>::new(0, order(17)), Err(io::Error::Order));
        assert_eq!(
            Fpage::<Io>::new(0x3f9, order(3)),
            Err(io::Error::Misaligned)
        );
        assert!(Fpage::<Io>::new(0, Order::WHOLE_IO_SPACE).is_ok());
        assert_eq!(
            Fpage::<Io>::new(1, Order::WHOLE_IO_SPACE),
            Err(io::Error::Misaligned)
        );
    }

    #[test]
    fn object_selector_and_alignment_are_checked() {
        let new = |slot, order| Fpage::<Object>::new(slot, order, CapRights::R);

        assert_eq!(
            new(CapIdx::INVALID, order(0)),
            Err(object::Error::InvalidCap)
        );
        assert_eq!(
            new(CapIdx::from_index(3), order(1)),
            Err(object::Error::Misaligned)
        );
        assert!(new(CapIdx::from_index(2), order(1)).is_ok());
        assert_eq!(
            new(CapIdx::from_index(0), order(63)),
            Err(object::Error::Order)
        );
    }

    #[test]
    fn flag_bits_of_a_selector_are_not_part_of_the_slot() {
        let selector = CapIdx::from_bits(CapIdx::from_index(5).bits() | 0xa);
        let page = Fpage::<Object>::new(selector, order(0), CapRights::R).unwrap();

        assert_eq!(page.cap(), CapIdx::from_index(5));
        assert_eq!(page.bits() & 0xfff, 0x4 | 3 << 4);
    }

    #[test]
    fn a_received_word_is_sorted_by_its_type() {
        let word = Fpage::<Memory>::new(0x1000, Order::PAGE, MemRights::RW)
            .unwrap()
            .bits();
        let any = Generic::from_bits(word);

        assert_eq!(any.kind(), Type::Memory);
        let page = Fpage::<Memory>::try_from(any).unwrap();
        assert_eq!(page.addr(), 0x1000);

        assert_eq!(Fpage::<Object>::try_from(any), Err(object::Error::Type));
    }

    #[test]
    fn only_valid_states_can_be_decoded() {
        let rights = MemRights::RW.bits();
        let page = Fpage::<Memory>::decode(rights, order(12), PayloadType::new(1)).unwrap();
        assert_eq!(page.bits(), 0x1316);

        // Below a page, no memory flexpage exists.
        assert_eq!(
            Fpage::<Memory>::decode(rights, order(11), PayloadType::new(1)),
            Err(memory::Error::Order)
        );

        // Special flexpages carry no rights and no payload.
        assert_eq!(
            Fpage::<Special>::decode(rights, Order::ZERO, PayloadType::new(0)),
            Err(special::Error::Rights)
        );
        assert_eq!(
            Fpage::<Special>::decode(RightsType::new(0), Order::ZERO, PayloadType::new(1)),
            Err(special::Error::Payload)
        );

        // The I/O port space is smaller than the address space.
        assert_eq!(
            Fpage::<Io>::decode(MemRights::RW.bits(), order(17), PayloadType::new(0),),
            Err(io::Error::Order)
        );
    }

    #[test]
    fn a_failed_conversion_keeps_the_word() {
        let word = Generic::from_bits((1 << 4) | (5 << 6));
        let failed = Fpage::<Memory>::try_from(word);

        assert_eq!(failed, Err(memory::Error::Order));
        assert_eq!(word.bits(), (1 << 4) | (5 << 6));
    }

    #[test]
    fn debug_shows_the_word() {
        extern crate std;
        use std::format;

        assert_eq!(format!("{:?}", Fpage::<Special>::all()), "Fpage(0xfc0)");
    }

    #[test]
    fn generic_debug_shows_the_kind() {
        extern crate std;
        use std::format;

        let memory = Generic::from_bits(0x1316);
        assert_eq!(
            format!("{memory:?}"),
            "Generic(Memory { page: Page(1), order: Order(12), rights: MemRights(W|R) })"
        );

        let special = Generic::from_bits(0xfc0);
        assert_eq!(
            format!("{special:?}"),
            "Generic(Special { order: Order(63) })"
        );

        // An order below a page is no memory flexpage, and the other kinds
        // do not fit either.
        let invalid = Generic::from_bits((1 << 4) | (5 << 6));
        assert_eq!(format!("{invalid:?}"), "Generic(Invalid(0x150))");
    }

    #[test]
    fn type_and_rights_fields_are_raw() {
        let page = Fpage::<Memory>::new(0x1000, Order::PAGE, MemRights::RW).unwrap();
        let memory = page.into_inner();

        assert_eq!(memory.rights_bits(), u4::new(6));
        assert_eq!(memory.order_bits().value(), 12);
        assert_eq!(Memory::TYPE.to_bits(), u2::new(1));
    }
}
