//! I/O flexpages: a range of I/O ports.
use super::layout::{OrderType, PayloadType, RightsType, TypeType};
use super::{Kind, MemRights, Order, Page, Type, aligned};
use l4_sys::umword;

/// An I/O flexpage: a range of `1 << order` I/O ports (`l4_iofpage`).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Io {
    port: u16,
    order: Order,
}

/// Why an I/O flexpage cannot be built or decoded.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Error {
    /// The order is bigger than the order of the whole I/O port space.
    Order,
    /// The port does not fit into the page field, or it is not a multiple
    /// of the size of the flexpage.
    Misaligned,
    /// The type field is not that of an I/O flexpage.
    Type,
}

impl Io {
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
        Ok(Io { port, order })
    }

    /// The first port.
    pub fn port(self) -> u16 {
        self.port
    }

    /// The size of the flexpage as a power of two.
    pub fn order(self) -> Order {
        self.order
    }

    /// The rights of the mapping: I/O ports are always mapped read-write.
    pub fn rights(self) -> MemRights {
        MemRights::RW
    }
}

impl Kind for Io {
    const TYPE: Type = Type::Io;
    type Error = Error;

    fn bits(&self) -> umword {
        MemRights::RW.bits().value() as umword
            | (Self::TYPE.to_bits().value() as umword) << 4
            | (self.order.get() as umword) << 6
            | (self.port as umword) << Page::SHIFT
    }

    fn rights_bits(&self) -> RightsType {
        MemRights::RW.bits()
    }

    fn order_bits(&self) -> OrderType {
        self.order.bits()
    }

    fn payload_bits(&self) -> PayloadType {
        Page::new(self.port as umword).payload()
    }

    fn decode(
        _rights: RightsType,
        r#type: TypeType,
        order: OrderType,
        payload: PayloadType,
    ) -> Result<Self, Error> {
        if r#type != Self::TYPE.to_bits() {
            return Err(Error::Type);
        }
        let order = Order::from_bits(order);
        if order > Order::WHOLE_IO_SPACE {
            return Err(Error::Order);
        }
        let port = u16::try_from(payload.value()).map_err(|_| Error::Misaligned)?;
        if !aligned(port.into(), order) {
            return Err(Error::Misaligned);
        }
        Ok(Io { port, order })
    }
}
