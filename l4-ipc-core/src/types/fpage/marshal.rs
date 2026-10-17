//! The wire format of flexpages.
use super::{Fpage, Generic, Kind};
use crate::Error;
use crate::protocol::{Marshal, Reader, Unmarshal, Writer};

/// A flexpage is sent as the one word it is.
impl<K: Kind> Marshal for Fpage<K> {
    fn marshal(&self, w: &mut Writer<'_>) -> Result<(), Error> {
        w.push(self.bits())
    }
}

/// A flexpage that is received has no kind yet, see [`Generic`].
impl Unmarshal for Generic {
    fn unmarshal(r: &mut Reader<'_>) -> Result<Self, Error> {
        r.pop().map(Generic::from_bits)
    }
}

#[cfg(all(test, not(target_os = "none")))]
mod tests {
    use super::super::{MemRights, Memory, Order};
    use super::*;
    use crate::message::{Tag, TagFlags};
    use l4_sys::Utcb;

    #[test]
    fn a_flexpage_travels_as_one_word() {
        let utcb = Utcb::zeroed();
        let page = Fpage::<Memory>::new(0x1000, Order::PAGE, MemRights::RW).unwrap();

        let mut w = Writer::new(&utcb);
        page.marshal(&mut w).unwrap();
        assert_eq!(w.words(), 1);
        assert_eq!(utcb.mr(0), page.bits());

        let tag = Tag::new(0, 1, 0, TagFlags::NONE);
        let received = Generic::unmarshal(&mut Reader::new(&utcb, tag)).unwrap();
        assert_eq!(received, Generic::from(page));
    }

    #[test]
    fn a_missing_word_is_truncated() {
        let utcb = Utcb::zeroed();
        let tag = Tag::new(0, 0, 0, TagFlags::NONE);

        assert_eq!(
            Generic::unmarshal(&mut Reader::new(&utcb, tag)),
            Err(Error::Truncated)
        );
    }
}
