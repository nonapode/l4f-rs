use crate::message::Tag;
use crate::Error;
use core::mem::size_of;
use l4_sys::{mword, umword, Utcb, MR_COUNT};

const WORD: usize = size_of::<umword>();

/// Sequentially writes untyped words into the message registers.
pub struct Writer<'u> {
    utcb: &'u Utcb,
    pos: usize,
}

impl<'u> Writer<'u> {
    /// Start writing at message register 0.
    pub fn new(utcb: &'u Utcb) -> Self {
        Writer { utcb, pos: 0 }
    }

    /// Append one word.
    pub fn push(&mut self, word: umword) -> Result<(), Error> {
        if self.pos >= MR_COUNT {
            return Err(Error::Overflow);
        }
        self.utcb.set_mr(self.pos, word);
        self.pos += 1;
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

    /// Number of words written so far.
    pub fn words(&self) -> usize {
        self.pos
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
