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
    /// A flexpage could not be built.
    Fpage(crate::types::fpage::Error),
}

impl From<crate::types::fpage::Error> for Error {
    fn from(error: crate::types::fpage::Error) -> Self {
        Error::Fpage(error)
    }
}

#[cfg(all(test, not(target_os = "none")))]
mod tests {
    use super::*;

    fn build() -> Result<(), Error> {
        Err(crate::types::fpage::Error::Misaligned)?
    }

    #[test]
    fn flexpage_errors_convert_with_the_question_mark() {
        assert_eq!(
            build(),
            Err(Error::Fpage(crate::types::fpage::Error::Misaligned))
        );
    }
}
