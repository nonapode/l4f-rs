use crate::Error;
use crate::message::Tag;
use core::mem::size_of;
use l4_sys::{MR_COUNT, Utcb, mword, umword};

const WORD: usize = size_of::<umword>();

/// Sequentially writes the untyped words, followed by the typed items, into
/// the message registers.
pub struct Writer<'u> {
    utcb: &'u Utcb,
    /// The next message register.
    pos: usize,
    /// The number of untyped words.
    words: usize,
    /// The number of typed items, which follow the untyped words.
    items: usize,
}

impl<'u> Writer<'u> {
    /// Start writing at message register 0.
    pub fn new(utcb: &'u Utcb) -> Self {
        Writer {
            utcb,
            pos: 0,
            words: 0,
            items: 0,
        }
    }

    /// Append one untyped word.  Fails with [`Error::Misordered`] once a
    /// typed item has been added.
    pub fn push(&mut self, word: umword) -> Result<(), Error> {
        if self.items > 0 {
            return Err(Error::Misordered);
        }
        if self.pos >= MR_COUNT {
            return Err(Error::Overflow);
        }
        self.utcb.set_mr(self.pos, word);
        self.pos += 1;
        self.words += 1;
        Ok(())
    }

    /// Append a typed item of two words, the control word and the flexpage.
    /// Nothing is written if they do not fit.
    pub fn push_item(&mut self, control: umword, fpage: umword) -> Result<(), Error> {
        if self.pos + 2 > MR_COUNT {
            return Err(Error::Overflow);
        }
        self.utcb.set_mr(self.pos, control);
        self.utcb.set_mr(self.pos + 1, fpage);
        self.pos += 2;
        self.items += 1;
        Ok(())
    }

    /// Append raw bytes, packed into words in memory order; the last word
    /// is zero-padded.
    pub fn push_bytes(&mut self, bytes: &[u8]) -> Result<(), Error> {
        for chunk in bytes.chunks(WORD) {
            let mut word = [0u8; WORD];
            word[..chunk.len()].copy_from_slice(chunk);
            self.push(umword::from_ne_bytes(word))?;
        }
        Ok(())
    }

    /// Number of untyped words written so far.
    pub fn words(&self) -> usize {
        self.words
    }

    /// Number of typed items written so far.
    pub fn items(&self) -> usize {
        self.items
    }
}

/// Sequentially reads the untyped words of a received message.
pub struct Reader<'u> {
    utcb: &'u Utcb,
    tag: Tag,
    pos: usize,
}

impl<'u> Reader<'u> {
    /// Read the words announced by `tag` from message register 0 on.
    pub fn new(utcb: &'u Utcb, tag: Tag) -> Self {
        Reader { utcb, tag, pos: 0 }
    }

    /// The label of the received message tag.
    pub fn label(&self) -> mword {
        self.tag.label()
    }

    /// Number of words not read yet.
    pub fn remaining(&self) -> usize {
        self.tag.words().saturating_sub(self.pos)
    }

    /// Read the next word.
    pub fn pop(&mut self) -> Result<umword, Error> {
        if self.remaining() == 0 {
            return Err(Error::Truncated);
        }
        let word = self.utcb.mr(self.pos);
        self.pos += 1;
        Ok(word)
    }

    /// Fill `out` with bytes packed by [`Writer::push_bytes`].
    pub fn pop_bytes(&mut self, out: &mut [u8]) -> Result<(), Error> {
        if self.remaining() < out.len().div_ceil(WORD) {
            return Err(Error::Truncated);
        }
        for chunk in out.chunks_mut(WORD) {
            let word = self.pop()?.to_ne_bytes();
            chunk.copy_from_slice(&word[..chunk.len()]);
        }
        Ok(())
    }
}

/// A value that can be written into the message registers.
pub trait Marshal {
    /// Append the value to the message.
    fn marshal(&self, w: &mut Writer<'_>) -> Result<(), Error>;
}

/// A value that can be read from the message registers.
pub trait Unmarshal: Sized {
    /// Read the value from the message.
    fn unmarshal(r: &mut Reader<'_>) -> Result<Self, Error>;
}

impl Marshal for umword {
    fn marshal(&self, w: &mut Writer<'_>) -> Result<(), Error> {
        w.push(*self)
    }
}

impl Unmarshal for umword {
    fn unmarshal(r: &mut Reader<'_>) -> Result<Self, Error> {
        r.pop()
    }
}

impl Marshal for mword {
    fn marshal(&self, w: &mut Writer<'_>) -> Result<(), Error> {
        w.push(*self as umword)
    }
}

impl Unmarshal for mword {
    fn unmarshal(r: &mut Reader<'_>) -> Result<Self, Error> {
        r.pop().map(|w| w as mword)
    }
}

macro_rules! impl_tuples {
    ( $( ( $( ( $T:ident $n:tt ) )* ) ),* $(,)? ) => { $(
        impl< $( $T: Marshal ),* > Marshal for ( $( $T, )* ) {
            #[allow(unused_variables)]
            fn marshal(&self, w: &mut Writer<'_>) -> Result<(), Error> {
                $( self.$n.marshal(w)?; )*
                Ok(())
            }
        }

        impl< $( $T: Unmarshal ),* > Unmarshal for ( $( $T, )* ) {
            #[allow(unused_variables)]
            fn unmarshal(r: &mut Reader<'_>) -> Result<Self, Error> {
                Ok(( $( <$T as Unmarshal>::unmarshal(r)?, )* ))
            }
        }
    )* };
}

impl_tuples! {
    (),
    ((A 0)),
    ((A 0)(B 1)),
    ((A 0)(B 1)(C 2)),
    ((A 0)(B 1)(C 2)(D 3)),
}

#[cfg(all(test, not(target_os = "none")))]
mod tests {
    use super::*;
    use crate::message::TagFlags;

    #[test]
    fn items_follow_the_untyped_words() {
        let utcb = Utcb::zeroed();
        let mut w = Writer::new(&utcb);

        w.push(10).unwrap();
        w.push(11).unwrap();
        w.push_item(0xc, 0xf).unwrap();
        w.push_item(0xd, 0xe).unwrap();

        assert_eq!((w.words(), w.items()), (2, 2));
        let registers: [umword; 6] = core::array::from_fn(|i| utcb.mr(i));
        assert_eq!(registers, [10, 11, 0xc, 0xf, 0xd, 0xe]);
    }

    #[test]
    fn an_untyped_word_cannot_follow_an_item() {
        let utcb = Utcb::zeroed();
        let mut w = Writer::new(&utcb);

        w.push_item(1, 2).unwrap();

        assert_eq!(w.push(3), Err(Error::Misordered));
        assert_eq!(w.push_bytes(&[1]), Err(Error::Misordered));
        assert_eq!((w.words(), w.items()), (0, 1));
        assert_eq!(utcb.mr(2), 0);
    }

    #[test]
    fn an_item_that_does_not_fit_is_not_written() {
        let utcb = Utcb::zeroed();
        let mut w = Writer::new(&utcb);

        for _ in 0..MR_COUNT - 1 {
            w.push(1).unwrap();
        }

        assert_eq!(w.push_item(0xc, 0xf), Err(Error::Overflow));
        assert_eq!(w.items(), 0);
        assert_eq!(utcb.mr(MR_COUNT - 1), 0);
        assert!(w.push(1).is_ok());
    }

    #[test]
    fn words_round_trip() {
        let utcb = Utcb::zeroed();
        let mut w = Writer::new(&utcb);
        (1 as umword, 2 as umword).marshal(&mut w).unwrap();
        assert_eq!(w.words(), 2);

        let tag = Tag::new(0, w.words(), 0, TagFlags::NONE);
        let mut r = Reader::new(&utcb, tag);
        let (a, b) = <(umword, umword)>::unmarshal(&mut r).unwrap();
        assert_eq!((a, b), (1, 2));
        assert_eq!(r.pop(), Err(Error::Truncated));
    }

    #[test]
    fn bytes_are_packed_in_memory_order() {
        let utcb = Utcb::zeroed();
        let mut w = Writer::new(&utcb);
        w.push_bytes(b"hello world").unwrap();
        assert_eq!(w.words(), 11usize.div_ceil(WORD));
        let mut first = [0u8; WORD];
        first.copy_from_slice(&b"hello wo"[..WORD]);
        assert_eq!(utcb.mr(0), umword::from_ne_bytes(first));

        let tag = Tag::new(0, w.words(), 0, TagFlags::NONE);
        let mut out = [0u8; 11];
        Reader::new(&utcb, tag).pop_bytes(&mut out).unwrap();
        assert_eq!(&out, b"hello world");
    }

    #[test]
    fn writer_overflows() {
        let utcb = Utcb::zeroed();
        let mut w = Writer::new(&utcb);
        for _ in 0..MR_COUNT {
            w.push(0).unwrap();
        }
        assert_eq!(w.push(0), Err(Error::Overflow));
    }
}
