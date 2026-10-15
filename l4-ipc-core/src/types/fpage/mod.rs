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
//! # Ok::<(), l4_ipc_core::types::fpage::Error>(())
//! ```
//!
//! ```compile_fail,E0308
//! use l4_ipc_core::types::fpage::{CapRights, Fpage, Memory, Order};
//!
//! // Capability rights do not mean anything for a memory flexpage.
//! let page = Fpage::<Memory>::new(0x1000, Order::PAGE, CapRights::RW);
//! ```

use crate::CapIdx;
use core::fmt;
use core::hash::{Hash, Hasher};
use core::marker::PhantomData;
use l4_bitfield::{Access, Bitfield, Field, Marker};
use l4_sys::umword;

mod error;
mod flags;
mod kind;
mod marshal;
mod order;
mod page;
mod rights;
mod snd_base;

pub use error::Error;
pub use kind::{Any, Io, Kind, Memory, NoRights, Obj, Special};
pub use order::Order;
pub use page::{Page, Type};
pub use rights::{CapRights, MemRights};
pub use snd_base::{
    Cacheability, ObjOptions, OffsetField, Options, OptionsField, SndBase, Transfer, TransferField,
};

/// A flexpage (`l4_fpage_t`) of the kind `K`.
pub struct Fpage<K = Any>(umword, PhantomData<fn() -> K>);

/// The rights field of a flexpage of the kind `K`.
pub type RightsField<K> = Field<Fpage<K>, <K as Kind>::Rights, 0, 4>;

/// The type field of a flexpage.
pub type TypeField<K> = Field<Fpage<K>, Type, 4, 6>;

/// The order field of a flexpage.
pub type OrderField<K> = Field<Fpage<K>, Order, 6, 12>;

/// The page field of a flexpage.
///
/// The bounds are literals: the solver of generic constant expressions
/// cannot evaluate `Page::SHIFT` and `umword::BITS` for a generic kind.
#[cfg(target_pointer_width = "64")]
pub type PageField<K> = Field<Fpage<K>, Page, 12, 64>;

/// The page field of a flexpage.
#[cfg(target_pointer_width = "32")]
pub type PageField<K> = Field<Fpage<K>, Page, 12, 32>;

const _: () = {
    assert!(Page::SHIFT == 12);
    assert!(umword::BITS == usize::BITS);
    assert!(Page::SHIFT + Page::BITS == umword::BITS as usize);
    assert!(if cfg!(target_pointer_width = "64") {
        umword::BITS == 64
    } else {
        umword::BITS == 32
    });
};

impl<K> Clone for Fpage<K> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<K> Copy for Fpage<K> {}

impl<K> PartialEq for Fpage<K> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<K> Eq for Fpage<K> {}

impl<K> Hash for Fpage<K> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

impl<K> fmt::Debug for Fpage<K> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Fpage")
            .field(&format_args!("{:#x}", self.0))
            .finish()
    }
}

impl<K> Bitfield for Fpage<K> {
    type Type = umword;

    fn from_raw(value: umword, _: Marker) -> Self {
        Fpage(value, PhantomData)
    }

    fn raw(&self, _: Marker) -> &umword {
        &self.0
    }
}

impl<K> Fpage<K> {
    /// The raw word, as it is stored in a message register.
    pub const fn bits(self) -> umword {
        self.0
    }

    /// Forget the kind.
    pub const fn into_any(self) -> Fpage<Any> {
        Fpage(self.0, PhantomData)
    }
}

impl<K: Kind> Fpage<K> {
    fn assemble(order: Order, rights: K::Rights, page: Page) -> Self {
        let mut fpage = Fpage(0, PhantomData);

        fpage
            .set::<TypeField<K>>(&K::TYPE)
            .set::<RightsField<K>>(&rights)
            .set::<OrderField<K>>(&order)
            .set::<PageField<K>>(&page);
        fpage
    }

    fn page(self) -> Page {
        self.get::<PageField<K>>()
    }
}

// The accessors are implemented for every kind instead of once for a generic
// kind: a generic method is compiled in the crate that calls it, which would
// need generic_const_exprs for the bounds of the fields as well.
macro_rules! accessors {
    ($($kind:ident),+) => {$(
        impl Fpage<$kind> {
            /// The size of the flexpage as a power of two.
            pub fn order(self) -> Order {
                self.get::<OrderField<$kind>>()
            }

            /// The rights of the flexpage.
            pub fn rights(self) -> <$kind as Kind>::Rights {
                self.get::<RightsField<$kind>>()
            }

            /// The same flexpage with other rights (`l4_fpage_set_rights`).
            #[must_use]
            pub fn with_rights(mut self, rights: <$kind as Kind>::Rights) -> Self {
                self.set::<RightsField<$kind>>(&rights);
                self
            }
        }
    )+};
}

accessors!(Special, Memory, Io, Obj);

impl Fpage<Any> {
    /// The size of the flexpage as a power of two.
    pub fn order(self) -> Order {
        self.get::<OrderField<Any>>()
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

impl Fpage<Memory> {
    /// The memory flexpage of `1 << order` bytes at `addr`
    /// (`l4_fpage`).
    ///
    /// The order cannot be smaller than a page, and the address has to be a
    /// multiple of the size.
    pub fn new(addr: umword, order: Order, rights: MemRights) -> Result<Self, Error> {
        if order < Order::PAGE {
            return Err(Error::Order);
        }
        if !aligned(addr, order) {
            return Err(Error::Misaligned);
        }
        Ok(Self::assemble(
            order,
            rights,
            Page::new(addr >> Page::SHIFT),
        ))
    }

    /// The address of the first byte.
    pub fn addr(self) -> umword {
        self.page().get() << Page::SHIFT
    }
}

impl Fpage<Io> {
    /// The I/O flexpage of `1 << order` ports at `port` (`l4_iofpage`).
    /// Its rights are read-write.
    ///
    /// The order is at most that of the whole I/O port space, and the port
    /// has to be a multiple of the size.
    pub fn new(port: u16, order: Order) -> Result<Self, Error> {
        if order > Order::WHOLE_IO_SPACE {
            return Err(Error::Order);
        }
        if !aligned(port.into(), order) {
            return Err(Error::Misaligned);
        }
        Ok(Self::assemble(order, MemRights::RW, Page::new(port.into())))
    }

    /// The first port.
    pub fn port(self) -> u16 {
        // The port space has 16 bits.
        self.page().get() as u16
    }
}

impl Fpage<Obj> {
    /// The capability flexpage of `1 << order` slots starting at `slot`
    /// (`l4_obj_fpage`).
    ///
    /// The selector has to be valid, the order cannot exceed the number of
    /// bits of a slot index, and the slot has to be a multiple of the size.
    pub fn new(slot: CapIdx, order: Order, rights: CapRights) -> Result<Self, Error> {
        if !slot.is_valid() {
            return Err(Error::InvalidCap);
        }
        if usize::from(order.get()) > Page::BITS {
            return Err(Error::Order);
        }
        if !aligned(slot.index(), order) {
            return Err(Error::Misaligned);
        }
        Ok(Self::assemble(order, rights, Page::new(slot.index())))
    }

    /// The selector of the first capability slot.
    pub fn cap(self) -> CapIdx {
        CapIdx::from_index(self.page().get())
    }
}

impl Fpage<Special> {
    /// The flexpage that is not valid (`l4_fpage_invalid`).
    pub const fn invalid() -> Self {
        Fpage(0, PhantomData)
    }

    /// The flexpage that covers all spaces (`l4_fpage_all`).
    pub const fn all() -> Self {
        Fpage((Order::WHOLE_SPACE.get() as umword) << 6, PhantomData)
    }
}

/// A flexpage of any kind, sorted by what its type field says.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Typed {
    Special(Fpage<Special>),
    Memory(Fpage<Memory>),
    Io(Fpage<Io>),
    Obj(Fpage<Obj>),
}

impl Fpage<Any> {
    /// The flexpage for a raw word, e.g. one received in a message.
    pub const fn from_bits(bits: umword) -> Self {
        Fpage(bits, PhantomData)
    }

    /// The kind according to the type field.
    pub fn kind(self) -> Type {
        self.get::<TypeField<Any>>()
    }

    /// Sort the flexpage by its kind.
    pub fn resolve(self) -> Typed {
        let bits = self.0;

        match self.kind() {
            Type::Special => Typed::Special(Fpage(bits, PhantomData)),
            Type::Memory => Typed::Memory(Fpage(bits, PhantomData)),
            Type::Io => Typed::Io(Fpage(bits, PhantomData)),
            Type::Obj => Typed::Obj(Fpage(bits, PhantomData)),
        }
    }

    /// The flexpage as the kind `K`, or itself if it is of another kind.
    pub fn try_into_kind<K: Kind>(self) -> Result<Fpage<K>, Self> {
        if self.kind() == K::TYPE {
            Ok(Fpage(self.0, PhantomData))
        } else {
            Err(self)
        }
    }
}

#[cfg(all(test, not(target_os = "none")))]
mod tests {
    use super::*;

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
        let page = Fpage::<Obj>::new(factory, order(0), CapRights::RWS).unwrap();

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
    fn assembling_agrees_with_the_constants() {
        let all = Fpage::<Special>::assemble(Order::WHOLE_SPACE, NoRights, Page::new(0));
        assert_eq!(all, Fpage::all());
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

        assert_eq!(new(0, order(11)), Err(Error::Order));
        assert_eq!(new(0x1000, order(13)), Err(Error::Misaligned));
        assert_eq!(new(0x1800, Order::PAGE), Err(Error::Misaligned));
        assert!(new(0x2000, order(13)).is_ok());
    }

    #[test]
    fn the_whole_address_space_starts_at_zero() {
        let new = |addr| Fpage::<Memory>::new(addr, Order::WHOLE_SPACE, MemRights::RWX);

        assert!(new(0).is_ok());
        assert_eq!(new(0x1000), Err(Error::Misaligned));
        assert_eq!(new(1 << 63), Err(Error::Misaligned));
    }

    #[test]
    fn io_order_and_alignment_are_checked() {
        assert_eq!(Fpage::<Io>::new(0, order(17)), Err(Error::Order));
        assert_eq!(Fpage::<Io>::new(0x3f9, order(3)), Err(Error::Misaligned));
        assert!(Fpage::<Io>::new(0, Order::WHOLE_IO_SPACE).is_ok());
        assert_eq!(
            Fpage::<Io>::new(1, Order::WHOLE_IO_SPACE),
            Err(Error::Misaligned)
        );
    }

    #[test]
    fn object_selector_and_alignment_are_checked() {
        let new = |slot, order| Fpage::<Obj>::new(slot, order, CapRights::R);

        assert_eq!(new(CapIdx::INVALID, order(0)), Err(Error::InvalidCap));
        assert_eq!(new(CapIdx::from_index(3), order(1)), Err(Error::Misaligned));
        assert!(new(CapIdx::from_index(2), order(1)).is_ok());
        assert_eq!(new(CapIdx::from_index(0), order(63)), Err(Error::Order));
    }

    #[test]
    fn flag_bits_of_a_selector_are_not_part_of_the_slot() {
        let selector = CapIdx::from_bits(CapIdx::from_index(5).bits() | 0xa);
        let page = Fpage::<Obj>::new(selector, order(0), CapRights::R).unwrap();

        assert_eq!(page.cap(), CapIdx::from_index(5));
        assert_eq!(page.bits() & 0xfff, 0x4 | 3 << 4);
    }

    #[test]
    fn a_received_word_is_sorted_by_its_type() {
        let word = Fpage::<Memory>::new(0x1000, Order::PAGE, MemRights::RW)
            .unwrap()
            .bits();
        let any = Fpage::from_bits(word);

        assert_eq!(any.kind(), Type::Memory);
        match any.resolve() {
            Typed::Memory(page) => assert_eq!(page.addr(), 0x1000),
            other => panic!("not a memory flexpage: {other:?}"),
        }

        assert_eq!(any.try_into_kind::<Obj>(), Err(any));
        assert_eq!(any.try_into_kind::<Memory>().unwrap().addr(), 0x1000);
    }

    #[test]
    fn every_type_value_resolves() {
        for ty in 0..4 {
            let any = Fpage::from_bits(ty << 4);
            assert!(matches!(
                (ty, any.resolve()),
                (0, Typed::Special(_))
                    | (1, Typed::Memory(_))
                    | (2, Typed::Io(_))
                    | (3, Typed::Obj(_))
            ));
        }
    }

    #[test]
    fn debug_shows_the_word() {
        extern crate std;
        use std::format;

        assert_eq!(format!("{:?}", Fpage::<Special>::all()), "Fpage(0xfc0)");
    }
}
