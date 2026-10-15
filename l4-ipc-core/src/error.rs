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
}
