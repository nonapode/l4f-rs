//! Variable arguments (`L4_VARG_TYPE_*`): self-describing values that
//! follow the fixed part of a request, e.g. the arguments of the factory's
//! create operation.
//!
//! Every argument starts with a header word that holds the type in its low
//! bits and the payload length in bytes from bit 16 on.  The payload
//! follows in whole words, the last one padded with zeros.

use super::{Marshal, Writer};
use crate::Error;
use crate::types::Fpage;
use crate::types::fpage::Kind;
use core::ffi::CStr;
use core::fmt;
use core::mem::size_of;
use l4_sys::{mword, umword};

const NIL: umword = 0x00;
const UMWORD: umword = 0x01;
const STRING: umword = 0x02;
const FPAGE: umword = 0x03;
const SIGN: umword = 0x80;
const MWORD: umword = UMWORD | SIGN;

/// Position of the payload length within the header word.
const LEN_SHIFT: u32 = 16;

fn header(kind: umword, len: usize) -> Result<umword, Error> {
    if len > umword::MAX >> LEN_SHIFT {
        return Err(Error::Overflow);
    }
    Ok(kind | (len as umword) << LEN_SHIFT)
}

/// An unsigned machine word (`L4_VARG_TYPE_UMWORD`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Uint(pub umword);

impl Marshal for Uint {
    fn marshal(&self, w: &mut Writer<'_>) -> Result<(), Error> {
        w.push(header(UMWORD, size_of::<umword>())?)?;
        w.push(self.0)
    }
}

/// A signed machine word (`L4_VARG_TYPE_MWORD`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Int(pub mword);

impl Marshal for Int {
    fn marshal(&self, w: &mut Writer<'_>) -> Result<(), Error> {
        w.push(header(MWORD, size_of::<mword>())?)?;
        w.push(self.0 as umword)
    }
}

/// The empty argument (`L4_VARG_TYPE_NIL`): a header and no payload.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Nil;

impl Marshal for Nil {
    fn marshal(&self, w: &mut Writer<'_>) -> Result<(), Error> {
        w.push(header(NIL, 0)?)
    }
}

/// A zero-terminated string (`L4_VARG_TYPE_STRING`).  The terminator is
/// part of the payload, like `l4_factory_create_add_str` sends it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Str<'a>(pub &'a CStr);

impl Marshal for Str<'_> {
    fn marshal(&self, w: &mut Writer<'_>) -> Result<(), Error> {
        let bytes = self.0.to_bytes_with_nul();
        w.push(header(STRING, bytes.len())?)?;
        w.push_bytes(bytes)
    }
}

/// A string of explicit length (`L4_VARG_TYPE_STRING`) without terminator,
/// like `l4_factory_create_add_lstr`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct LStr<'a>(pub &'a [u8]);

impl Marshal for LStr<'_> {
    fn marshal(&self, w: &mut Writer<'_>) -> Result<(), Error> {
        w.push(header(STRING, self.0.len())?)?;
        w.push_bytes(self.0)
    }
}

/// A flexpage (`L4_VARG_TYPE_FPAGE`).
pub struct Flexpage<K: Kind>(pub Fpage<K>);

impl<K: Kind> Clone for Flexpage<K> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<K: Kind> Copy for Flexpage<K> {}

impl<K: Kind> fmt::Debug for Flexpage<K> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Flexpage").field(&self.0).finish()
    }
}

impl<K: Kind> Marshal for Flexpage<K> {
    fn marshal(&self, w: &mut Writer<'_>) -> Result<(), Error> {
        w.push(header(FPAGE, size_of::<umword>())?)?;
        self.0.marshal(w)
    }
}

#[cfg(all(test, not(target_os = "none")))]
mod tests {
    use super::*;
    use l4_sys::{MR_COUNT, Utcb};

    const W: usize = size_of::<umword>();

    /// Marshal `arg` into a fresh UTCB; returns it with the number of words
    /// written.
    fn encode(arg: &impl Marshal) -> (Utcb, Result<usize, Error>) {
        let utcb = Utcb::zeroed();
        let mut w = Writer::new(&utcb);
        let words = arg.marshal(&mut w).map(|()| w.words());
        (utcb, words)
    }

    fn word(bytes: &[u8]) -> umword {
        let mut word = [0u8; W];
        word[..bytes.len()].copy_from_slice(bytes);
        umword::from_ne_bytes(word)
    }

    #[test]
    fn unsigned_word() {
        let (utcb, words) = encode(&Uint(5));
        assert_eq!(words, Ok(2));
        assert_eq!(utcb.mr(0), 0x01 | (W as umword) << 16);
        assert_eq!(utcb.mr(1), 5);
    }

    #[test]
    fn signed_word() {
        let (utcb, words) = encode(&Int(-2));
        assert_eq!(words, Ok(2));
        assert_eq!(utcb.mr(0), 0x81 | (W as umword) << 16);
        assert_eq!(utcb.mr(1), -2isize as umword);
    }

    #[test]
    fn nil_is_a_bare_header() {
        let (utcb, words) = encode(&Nil);
        assert_eq!(words, Ok(1));
        assert_eq!(utcb.mr(0), 0);
    }

    #[test]
    fn string_carries_its_terminator() {
        let (utcb, words) = encode(&Str(c"hi"));
        assert_eq!(words, Ok(2));
        assert_eq!(utcb.mr(0), 0x02 | 3 << 16);
        assert_eq!(utcb.mr(1), word(b"hi"));
    }

    #[test]
    fn terminator_may_need_a_word_of_its_own() {
        let mut bytes = [b'x'; W + 1];
        bytes[W] = 0;
        let (utcb, words) = encode(&Str(CStr::from_bytes_with_nul(&bytes).unwrap()));
        assert_eq!(words, Ok(3));
        assert_eq!(utcb.mr(0), 0x02 | ((W + 1) as umword) << 16);
        assert_eq!(utcb.mr(1), word(&[b'x'; W]));
        assert_eq!(utcb.mr(2), 0);
    }

    #[test]
    fn explicit_length_string_has_no_terminator() {
        let (utcb, words) = encode(&LStr(b"abc"));
        assert_eq!(words, Ok(2));
        assert_eq!(utcb.mr(0), 0x02 | 3 << 16);
        assert_eq!(utcb.mr(1), word(b"abc"));

        let (utcb, words) = encode(&LStr(b""));
        assert_eq!(words, Ok(1));
        assert_eq!(utcb.mr(0), 0x02);
    }

    #[test]
    fn flexpage_is_a_word_after_a_header() {
        use crate::types::fpage::{Fpage, Special};

        let (utcb, words) = encode(&Flexpage(Fpage::<Special>::all()));
        assert_eq!(words, Ok(2));
        assert_eq!(utcb.mr(0), 0x03 | (W as umword) << 16);
        assert_eq!(utcb.mr(1), 63 << 6);
    }

    #[test]
    fn arguments_follow_each_other() {
        let (utcb, words) = encode(&(Uint(1), Nil, Int(-1)));
        assert_eq!(words, Ok(5));
        assert_eq!(utcb.mr(1), 1);
        assert_eq!(utcb.mr(2), 0);
        assert_eq!(utcb.mr(4), umword::MAX);
    }

    #[test]
    fn payload_that_does_not_fit_overflows() {
        let (_, words) = encode(&LStr(&[0; MR_COUNT * W]));
        assert_eq!(words, Err(Error::Overflow));
    }
}
