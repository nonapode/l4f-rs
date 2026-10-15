use core::mem::size_of;
use l4_ipc_core::sys::{mword, umword, MR_COUNT};
use l4_ipc_core::protocol::{Marshal, Protocol, Writer, Operation, Invocation};
use l4_ipc_core::message::{TagFlags};
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

/// `L4_LOG_WRITE_OP`: write bytes to the console.  The kernel does not
/// answer, so the operation only sends and asks to be scheduled.
pub struct Write;

impl Operation for Write {
    type Proto = Log;
    const OPCODE: Option<umword> = Some(0);
    const INVOCATION: Invocation = Invocation::Send;
    const TAG_FLAGS: TagFlags = TagFlags::SCHEDULE;
    type Request<'a> = Bytes<'a>;
    type Reply = ();
}
