use l4_sys::{mword, umword, MR_COUNT};
use l4_ipc_core::protocol::{Marshal, Protocol, Writer};
use l4_ipc_core::Error;

/// The log protocol marker (`L4_PROTO_LOG`).
pub struct Log;

impl Log {
    /// Maximum number of bytes one write operation transfers
    /// (`L4_VCON_WRITE_SIZE`): all message registers except opcode and size.
    pub const WRITE_SIZE: usize = (MR_COUNT - 2) * size_of::<umword>();
}

impl Protocol for Log {
    const PROTO: mword = -13;
}

/// A byte string to write.  Its length must not exceed [`Log::WRITE_SIZE`].
pub struct Bytes<'a>(pub &'a [u8]);

impl Marshal for Bytes<'_> {
    fn marshal(&self, w: &mut Writer<'_>) -> Result<(), Error> {
        w.push(self.0.len() as umword)?;
        w.push_bytes(self.0)
    }
}
