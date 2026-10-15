use crate::Error;
use crate::protocol::{Marshal, Writer};
use crate::types::fpage::{Fpage, SndBase};
use l4_sys::umword;

/// `L4_ITEM_MAP`: the item is a map item.
pub(crate) const ITEM_MAP: umword = 8;

/// A map item: the kernel maps the flexpage of the sender into the space
/// of the receiver, or grants it, at the send base.
///
/// A map item is a typed item and follows the untyped words of a message.
pub struct MapItem<K> {
    /// Where the mapping is placed, and how.
    pub base: SndBase,
    /// What is mapped.
    pub fpage: Fpage<K>,
}

impl<K> Clone for MapItem<K> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<K> Copy for MapItem<K> {}

impl<K> MapItem<K> {
    pub const fn new(fpage: Fpage<K>, base: SndBase) -> MapItem<K> {
        MapItem { base, fpage }
    }

    /// The control word of the item (`l4_map_control`): the send base with
    /// the flag that marks a map item.
    pub const fn control(&self) -> umword {
        self.base.bits() | ITEM_MAP
    }
}

impl<K> Marshal for MapItem<K> {
    fn marshal(&self, w: &mut Writer<'_>) -> Result<(), Error> {
        w.push_item(self.control(), self.fpage.bits())
    }
}

#[cfg(all(test, not(target_os = "none")))]
mod tests {
    use super::*;
    use crate::CapIdx;
    use crate::types::fpage::{CapRights, Obj, Order, Transfer};
    use l4_sys::Utcb;

    fn slot_2() -> Fpage<Obj> {
        Fpage::<Obj>::new(
            CapIdx::from_index(2),
            Order::new(0).unwrap(),
            CapRights::RWS,
        )
        .unwrap()
    }

    #[test]
    fn a_map_item_is_a_control_word_and_a_flexpage() {
        let utcb = Utcb::zeroed();
        let item = MapItem::new(slot_2(), SndBase::default());

        let mut w = Writer::new(&utcb);
        item.marshal(&mut w).unwrap();

        // l4_map_obj_control(0, 0) and l4_obj_fpage(slot 2, 0, RWS)
        assert_eq!(utcb.mr(0), 8);
        assert_eq!(utcb.mr(1), 0x2037);
        assert_eq!((w.words(), w.items()), (0, 1));
    }

    #[test]
    fn the_control_word_keeps_the_send_base() {
        let base = SndBase::new(0x5000).unwrap().with_transfer(Transfer::Grant);
        let item = MapItem::new(slot_2(), base);

        assert_eq!(item.control(), 0x5000 | 2 | 8);
    }
}
