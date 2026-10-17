//! Memory flexpages: a range of memory.
use super::layout::{OrderType, PayloadType, RightsType, TypeType};
use super::{Kind, MemRights, Order, Page, Type, aligned};
use l4_sys::umword;

/// A memory flexpage: a range of `1 << order` bytes (`l4_fpage`).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Memory {
    page: Page,
    order: Order,
    rights: MemRights,
}

/// Why a memory flexpage cannot be built or decoded.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Error {
    /// The order is smaller than the order of a single page.
    Order,
    /// The address is not a multiple of the size of the flexpage.
    Misaligned,
    /// The type field is not that of a memory flexpage.
    Type,
}

impl Memory {
    /// The memory flexpage of `1 << order` bytes at `addr`
    /// (`l4_fpage`).
    ///
    /// The order cannot be smaller than a page, and the address has to be
    /// a multiple of the size.
    pub fn new(addr: umword, order: Order, rights: MemRights) -> Result<Self, Error> {
        if order < Order::PAGE {
            return Err(Error::Order);
        }
        if !aligned(addr, order) {
            return Err(Error::Misaligned);
        }
        Ok(Memory {
            page: Page::new(addr >> Page::SHIFT),
            order,
            rights,
        })
    }

    /// The address of the first byte.
    pub fn addr(self) -> umword {
        self.page.get() << Page::SHIFT
    }

    /// The size of the flexpage as a power of two.
    pub fn order(self) -> Order {
        self.order
    }

    /// The rights of the mapping.
    pub fn rights(self) -> MemRights {
        self.rights
    }

    /// The same flexpage with other rights (`l4_fpage_set_rights`).
    #[must_use]
    pub fn with_rights(mut self, rights: MemRights) -> Self {
        self.rights = rights;
        self
    }
}

impl Kind for Memory {
    const TYPE: Type = Type::Memory;
    type Error = Error;

    fn bits(&self) -> umword {
        self.rights.bits().value() as umword
            | (Self::TYPE.to_bits().value() as umword) << 4
            | (self.order.get() as umword) << 6
            | self.page.get() << Page::SHIFT
    }

    fn rights_bits(&self) -> RightsType {
        self.rights.bits()
    }

    fn order_bits(&self) -> OrderType {
        self.order.bits()
    }

    fn payload_bits(&self) -> PayloadType {
        self.page.payload()
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
        if order < Order::PAGE.bits() {
            return Err(Error::Order);
        }
        let order = Order::from_bits(order);
        let page = Page::from_payload(payload);
        if !aligned(page.get() << Page::SHIFT, order) {
            return Err(Error::Misaligned);
        }
        Ok(Memory {
            page,
            order,
            rights: MemRights::from_bits(rights),
        })
    }
}
