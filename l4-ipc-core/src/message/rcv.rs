use super::item::ITEM_MAP;
use crate::CapIdx;
use l4_sys::umword;

/// A receive item that accepts a single object capability
/// (`L4_RCV_ITEM_SINGLE_CAP`).
///
/// It is written into a buffer register before an IPC.  When the message
/// carries a map item, the kernel maps the capability into the slot the
/// receive item names.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct RcvCap(umword);

impl RcvCap {
    /// `L4_RCV_ITEM_SINGLE_CAP`: the receive window is one capability slot,
    /// which is stored in the upper bits of the item.
    const SINGLE_CAP: umword = ITEM_MAP | 2;

    /// Accept one capability and store it in `slot`.
    pub const fn new(slot: CapIdx) -> RcvCap {
        RcvCap(slot.bits() | Self::SINGLE_CAP)
    }

    /// The raw value for a buffer register.
    pub const fn bits(self) -> umword {
        self.0
    }
}

#[cfg(all(test, not(target_os = "none")))]
mod tests {
    use super::*;

    #[test]
    fn slot_and_flags_share_the_item() {
        let item = RcvCap::new(CapIdx::from_index(5));
        assert_eq!(item.bits() >> CapIdx::SHIFT, 5);
        assert_eq!(item.bits() & ((1 << CapIdx::SHIFT) - 1), 0xa);
    }
}
