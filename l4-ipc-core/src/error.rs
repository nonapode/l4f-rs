use crate::types::fpage;
use l4_sys::{mword, umword};

/// Errors of IPC based operations.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Error {
    /// The kernel flagged an IPC error; carries the raw error code from the
    /// UTCB's thread control registers.
    Ipc(umword),
    /// The receiver replied with a negative label, i.e. a protocol-level
    /// error code (`-L4_E*`).
    Proto(mword),
    /// The message does not fit into the message registers.
    Overflow,
    /// The message is shorter than expected.
    Truncated,
    /// An untyped word was added after a typed item.  The untyped words of
    /// a message come first.
    Misordered,
    /// A flexpage could not be built or decoded.
    Fpage(fpage::Error),
}

impl From<fpage::Error> for Error {
    fn from(error: fpage::Error) -> Self {
        Error::Fpage(error)
    }
}

macro_rules! from_kind_error {
    ($($kind:ident),+ $(,)?) => {$(
        impl From<fpage::$kind::Error> for Error {
            fn from(error: fpage::$kind::Error) -> Self {
                Error::Fpage(error.into())
            }
        }
    )+};
}

from_kind_error!(memory, io, object, special, snd_base);

#[cfg(all(test, not(target_os = "none")))]
mod tests {
    use super::*;
    use crate::types::fpage::memory;

    fn build() -> Result<(), Error> {
        Err(memory::Error::Misaligned)?
    }

    #[test]
    fn flexpage_errors_convert_with_the_question_mark() {
        assert_eq!(
            build(),
            Err(Error::Fpage(fpage::Error::Memory(
                memory::Error::Misaligned
            )))
        );
    }
}
