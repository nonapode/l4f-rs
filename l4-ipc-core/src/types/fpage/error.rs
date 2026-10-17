//! Why a flexpage cannot be built or decoded.
use super::{io, memory, object, snd_base, special};

/// Why a flexpage cannot be built or decoded.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Error {
    /// A memory flexpage cannot be built or decoded.
    Memory(memory::Error),
    /// An I/O flexpage cannot be built or decoded.
    Io(io::Error),
    /// A capability flexpage cannot be built or decoded.
    Object(object::Error),
    /// A special flexpage cannot be decoded.
    Special(special::Error),
    /// A send base cannot be built.
    SndBase(snd_base::Error),
}

impl From<memory::Error> for Error {
    fn from(error: memory::Error) -> Self {
        Error::Memory(error)
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::Io(error)
    }
}

impl From<object::Error> for Error {
    fn from(error: object::Error) -> Self {
        Error::Object(error)
    }
}

impl From<special::Error> for Error {
    fn from(error: special::Error) -> Self {
        Error::Special(error)
    }
}

impl From<snd_base::Error> for Error {
    fn from(error: snd_base::Error) -> Self {
        Error::SndBase(error)
    }
}
