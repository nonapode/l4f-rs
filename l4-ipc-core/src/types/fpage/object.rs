//! Capability flexpages: a range of capability slots.
use super::layout::{OrderType, PayloadType, RightsType, TypeType};
use super::{CapRights, Kind, Order, Page, Type, aligned};
use crate::CapIdx;
use l4_sys::umword;

/// A capability flexpage: a range of `1 << order` capability slots
/// (`l4_obj_fpage`).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Object {
    slot: CapIdx,
    order: Order,
    rights: CapRights,
}

/// Why a capability flexpage cannot be built or decoded.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Error {
    /// The capability selector is invalid.
    InvalidCap,
    /// The order is bigger than the number of bits of a slot index.
    Order,
    /// The slot index is not a multiple of the size of the flexpage.
    Misaligned,
    /// The type field is not that of a capability flexpage.
    Type,
}

impl Object {
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
        Ok(Object {
            slot: CapIdx::from_index(slot.index()),
            order,
            rights,
        })
    }

    /// The selector of the first capability slot.
    pub fn cap(self) -> CapIdx {
        self.slot
    }

    /// The size of the flexpage as a power of two.
    pub fn order(self) -> Order {
        self.order
    }

    /// The rights of the mapped capability.
    pub fn rights(self) -> CapRights {
        self.rights
    }

    /// The same flexpage with other rights (`l4_fpage_set_rights`).
    #[must_use]
    pub fn with_rights(mut self, rights: CapRights) -> Self {
        self.rights = rights;
        self
    }
}

impl Kind for Object {
    const TYPE: Type = Type::Obj;
    type Error = Error;

    fn bits(&self) -> umword {
        self.rights.bits().value() as umword
            | (Self::TYPE.to_bits().value() as umword) << 4
            | (self.order.get() as umword) << 6
            | (self.slot.index() as umword) << Page::SHIFT
    }

    fn rights_bits(&self) -> RightsType {
        self.rights.bits()
    }

    fn order_bits(&self) -> OrderType {
        self.order.bits()
    }

    fn payload_bits(&self) -> PayloadType {
        Page::new(self.slot.index()).payload()
    }

    fn decode(
        rights: RightsType,
        r#type: TypeType,
        order: OrderType,
        payload: PayloadType,
    ) -> Result<Self, Error> {
        if r#type != Self::TYPE.to_bits() {
            return Err(Error::Type);
        }
        let order = Order::from_bits(order);
        if usize::from(order.get()) > Page::BITS {
            return Err(Error::Order);
        }
        let page = Page::from_payload(payload);
        if !aligned(page.get() << Page::SHIFT, order) {
            return Err(Error::Misaligned);
        }
        Ok(Object {
            slot: CapIdx::from_index(page.get()),
            order,
            rights: CapRights::from_bits(rights),
        })
    }
}
