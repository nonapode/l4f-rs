use super::flags::flags;
use super::{Error, Page};
use core::fmt;
use l4_bitfield::{Access, Bitfield, Bits, Decode, Encode, Field, Marker};
use l4_sys::umword;

/// What happens to the mapping in the sender's space.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default, Debug)]
pub enum Transfer {
    /// The sender keeps its mapping (`L4_MAP_ITEM_MAP`).
    #[default]
    Map = 0,
    /// The mapping moves to the receiver (`L4_MAP_ITEM_GRANT`).
    Grant = 1,
}

impl Encode<umword, 1> for Transfer {
    fn encode(&self) -> Bits<umword, 1> {
        Bits::new(*self as umword)
    }
}

impl Decode<umword, 1> for Transfer {
    fn decode(bits: Bits<umword, 1>) -> Self {
        match bits.into_inner() {
            0 => Transfer::Map,
            _ => Transfer::Grant,
        }
    }
}

/// How the receiver of a memory mapping caches it.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default, Debug)]
pub enum Cacheability {
    /// Keep the attributes of the sender.
    #[default]
    Inherit = 0,
    /// Not cached (`L4_FPAGE_UNCACHEABLE`).
    Uncacheable = 1,
    /// Cached (`L4_FPAGE_CACHEABLE`).
    Cacheable = 3,
    /// Not cached, but writes may be buffered (`L4_FPAGE_BUFFERABLE`).
    Bufferable = 5,
}

impl Cacheability {
    const fn from_bits(bits: u8) -> Option<Cacheability> {
        match bits {
            0 => Some(Cacheability::Inherit),
            1 => Some(Cacheability::Uncacheable),
            3 => Some(Cacheability::Cacheable),
            5 => Some(Cacheability::Bufferable),
            _ => None,
        }
    }
}

flags! {
    /// The options of a mapping of capabilities (`L4_FPAGE_C_*`).
    ObjOptions {
        /// The mapping does not count as a reference of the object
        /// (`L4_FPAGE_C_NO_REF_CNT`).
        NO_REF_CNT = 1,
        /// An object specific right.
        RIGHT1 = 2,
        /// An object specific right.
        RIGHT2 = 4,
        /// An object specific right.
        RIGHT3 = 8,
    }
}

impl ObjOptions {
    /// The capability of an IPC gate has the server right
    /// (`L4_FPAGE_C_IPCGATE_SVR`).
    pub const IPC_GATE_SERVER: ObjOptions = ObjOptions::RIGHT1;
}

/// The four option bits of a [`SndBase`], which mean the cacheability for
/// memory and [`ObjOptions`] for capabilities.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default, Debug)]
pub struct Options(u8);

impl Encode<umword, 4> for Options {
    fn encode(&self) -> Bits<umword, 4> {
        Bits::new(self.0 as umword)
    }
}

impl Decode<umword, 4> for Options {
    fn decode(bits: Bits<umword, 4>) -> Self {
        Options(bits.into_inner() as u8)
    }
}

/// The send base of a mapping: where in the receive window the mapping is
/// placed, together with the control bits of the map item
/// (`l4_map_control`):
///
/// | bits    | field                                                 |
/// |---------|-------------------------------------------------------|
/// | `1`     | [`Transfer`]: map or grant                            |
/// | `4..8`  | options: [`Cacheability`] or [`ObjOptions`]           |
/// | `12..`  | the offset, in pages (or capability slots)            |
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct SndBase(umword);

/// The transfer field of a send base.
pub type TransferField = Field<SndBase, Transfer, 1, 2>;

/// The option bits of a send base.
pub type OptionsField = Field<SndBase, Options, 4, 8>;

/// The offset field of a send base.
#[cfg(target_pointer_width = "64")]
pub type OffsetField = Field<SndBase, Page, 12, 64>;

/// The offset field of a send base.
#[cfg(target_pointer_width = "32")]
pub type OffsetField = Field<SndBase, Page, 12, 32>;

impl fmt::Debug for SndBase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("SndBase")
            .field(&format_args!("{:#x}", self.0))
            .finish()
    }
}

impl Bitfield for SndBase {
    type Type = umword;

    fn from_raw(value: umword, _: Marker) -> Self {
        SndBase(value)
    }

    fn raw(&self, _: Marker) -> &umword {
        &self.0
    }
}

impl SndBase {
    /// The send base for the raw word.
    pub const fn from_bits(bits: umword) -> SndBase {
        SndBase(bits)
    }

    /// The raw word, as it is stored in a message register.
    pub const fn bits(self) -> umword {
        self.0
    }

    /// The send base at `offset` bytes (or capability selector bits) into
    /// the receive window.  The offset has to be a multiple of a page.
    pub fn new(offset: umword) -> Result<SndBase, Error> {
        if offset & ((1 << Page::SHIFT) - 1) != 0 {
            return Err(Error::Misaligned);
        }
        let mut base = SndBase(0);
        base.set::<OffsetField>(&Page::new(offset >> Page::SHIFT));
        Ok(base)
    }

    /// The offset into the receive window.
    pub fn offset(self) -> umword {
        self.get::<OffsetField>().get() << Page::SHIFT
    }

    /// Whether the mapping is moved or copied.
    pub fn transfer(self) -> Transfer {
        self.get::<TransferField>()
    }

    #[must_use]
    pub fn with_transfer(mut self, transfer: Transfer) -> SndBase {
        self.set::<TransferField>(&transfer);
        self
    }

    /// The options as the cacheability of a memory mapping; `None` if the
    /// bits are not one.
    pub fn cacheability(self) -> Option<Cacheability> {
        Cacheability::from_bits(self.get::<OptionsField>().0)
    }

    /// The same send base for a memory mapping with the cacheability.
    #[must_use]
    pub fn with_cacheability(mut self, cacheability: Cacheability) -> SndBase {
        self.set::<OptionsField>(&Options(cacheability as u8));
        self
    }

    /// The options as those of a mapping of capabilities.
    pub fn obj_options(self) -> ObjOptions {
        ObjOptions::from_bits(self.get::<OptionsField>().0).unwrap_or_default()
    }

    /// The same send base for a mapping of capabilities with the options.
    #[must_use]
    pub fn with_obj_options(mut self, options: ObjOptions) -> SndBase {
        self.set::<OptionsField>(&Options(options.bits()));
        self
    }
}

#[cfg(all(test, not(target_os = "none")))]
mod tests {
    use super::*;

    // The expected words are what l4_map_control computes without the
    // item flag: base | cache << 4 | grant.

    #[test]
    fn layout_matches_l4_map_control() {
        let base = SndBase::new(0x5000)
            .unwrap()
            .with_cacheability(Cacheability::Cacheable)
            .with_transfer(Transfer::Grant);

        assert_eq!(base.bits(), 0x5000 | 3 << 4 | 2);
        assert_eq!(base.offset(), 0x5000);
        assert_eq!(base.cacheability(), Some(Cacheability::Cacheable));
        assert_eq!(base.transfer(), Transfer::Grant);
    }

    #[test]
    fn object_options_use_the_same_bits() {
        let base = SndBase::default().with_obj_options(ObjOptions::IPC_GATE_SERVER);

        assert_eq!(base.bits(), 0x20);
        assert_eq!(base.obj_options(), ObjOptions::RIGHT1);
        assert_eq!(base.transfer(), Transfer::Map);

        let no_ref = SndBase::default().with_obj_options(ObjOptions::NO_REF_CNT);
        assert_eq!(no_ref.bits(), 0x10);
    }

    #[test]
    fn default_is_a_plain_map_at_zero() {
        assert_eq!(SndBase::default().bits(), 0);
        assert_eq!(SndBase::default().offset(), 0);
        assert_eq!(
            SndBase::default().cacheability(),
            Some(Cacheability::Inherit)
        );
    }

    #[test]
    fn the_offset_is_a_multiple_of_a_page() {
        assert_eq!(SndBase::new(0x1800), Err(Error::Misaligned));
        assert!(SndBase::new(0x2000).is_ok());
    }

    #[test]
    fn setting_a_field_keeps_the_others() {
        let base = SndBase::new(0x3000).unwrap().with_transfer(Transfer::Grant);
        let base = base.with_cacheability(Cacheability::Bufferable);

        assert_eq!(base.offset(), 0x3000);
        assert_eq!(base.transfer(), Transfer::Grant);
        assert_eq!(base.with_transfer(Transfer::Map).bits(), 0x3000 | 5 << 4);
    }

    #[test]
    fn unknown_option_bits_are_no_cacheability() {
        let base = SndBase::from_bits(2 << 4);

        assert_eq!(base.cacheability(), None);
        assert_eq!(base.obj_options(), ObjOptions::RIGHT1);
    }
}
