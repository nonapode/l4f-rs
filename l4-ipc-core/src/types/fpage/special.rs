//! Special flexpages: a range of the address space without memory behind it.
use super::layout::{OrderType, PayloadType, RightsType, TypeType};
use super::{Kind, NoRights, Order, Type};
use l4_sys::umword;

/// A special flexpage: a range of the address space, e.g. to unmap it or
/// to describe a whole space.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Special {
    order: Order,
}

/// Why a special flexpage cannot be decoded.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Error {
    /// The type field is not that of a special flexpage.
    Type,
    /// The rights field is not empty.
    Rights,
    /// The page field is not empty.
    Payload,
}

impl Special {
    /// The flexpage that is not valid (`l4_fpage_invalid`).
    pub const fn invalid() -> Self {
        Special { order: Order::ZERO }
    }

    /// The flexpage that covers all spaces (`l4_fpage_all`).
    pub const fn all() -> Self {
        Special {
            order: Order::WHOLE_SPACE,
        }
    }

    /// The size of the range as a power of two.
    pub fn order(self) -> Order {
        self.order
    }

    /// The rights of a special flexpage: none.
    pub fn rights(self) -> NoRights {
        NoRights
    }
}

impl Kind for Special {
    const TYPE: Type = Type::Special;
    type Error = Error;

    fn bits(&self) -> umword {
        (Self::TYPE.to_bits().value() as umword) << 4 | (self.order.get() as umword) << 6
    }

    fn rights_bits(&self) -> RightsType {
        RightsType::new(0)
    }

    fn order_bits(&self) -> OrderType {
        self.order.bits()
    }

    fn payload_bits(&self) -> PayloadType {
        PayloadType::new(0)
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
        if rights.value() != 0 {
            return Err(Error::Rights);
        }
        if payload.value() != 0 {
            return Err(Error::Payload);
        }
        Ok(Special {
            order: Order::from_bits(order),
        })
    }
}
