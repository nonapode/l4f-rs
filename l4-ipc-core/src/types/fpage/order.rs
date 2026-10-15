use l4_bitfield::{Bits, Decode, Encode};
use l4_sys::umword;

/// The size of a flexpage as a power of two (`order`), a six bit value.
///
/// For memory the size is the log2 of the number of bytes, for I/O ports of
/// the number of ports and for capabilities of the number of capability
/// slots.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Order(u8);

impl Order {
    /// The size of a memory page (`L4_FPAGE_ADDR_SHIFT`).
    pub const PAGE: Order = Order(12);

    /// The size of the whole I/O port space (`L4_WHOLE_IOADDRESS_SPACE`).
    pub const WHOLE_IO_SPACE: Order = Order(16);

    /// The size that covers the whole address space
    /// (`L4_WHOLE_ADDRESS_SPACE`).  It is also the size of the special
    /// flexpage that covers all spaces.
    pub const WHOLE_SPACE: Order = Order(63);

    /// The order `order`, if it fits into the six bits of the size field.
    pub const fn new(order: u8) -> Option<Order> {
        if order <= Self::WHOLE_SPACE.0 {
            Some(Order(order))
        } else {
            None
        }
    }

    pub const fn get(self) -> u8 {
        self.0
    }
}

impl Encode<umword, 6> for Order {
    fn encode(&self) -> Bits<umword, 6> {
        Bits::new(self.0 as umword)
    }
}

impl Decode<umword, 6> for Order {
    fn decode(bits: Bits<umword, 6>) -> Self {
        // All 64 values of the field are valid.
        Order(bits.into_inner() as u8)
    }
}

#[cfg(all(test, not(target_os = "none")))]
mod tests {
    use super::*;

    #[test]
    fn orders_have_six_bits() {
        assert_eq!(Order::new(0), Some(Order(0)));
        assert_eq!(Order::new(63), Some(Order::WHOLE_SPACE));
        assert_eq!(Order::new(64), None);
    }

    #[test]
    fn well_known_orders() {
        assert_eq!(Order::PAGE.get(), 12);
        assert_eq!(Order::WHOLE_IO_SPACE.get(), 16);
        assert_eq!(Order::WHOLE_SPACE.get(), 63);
    }

    #[test]
    fn codec_round_trips_every_order() {
        for order in 0..64 {
            let order = Order::new(order).unwrap();
            assert_eq!(Order::decode(order.encode()), order);
        }
    }
}
