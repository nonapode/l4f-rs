use l4_ipc_core::message::TagFlags;
use l4_ipc_core::protocol::{Operation, Invocation};
use l4_sys::umword;
/// Operations of the log protocol.
use super::*;

/// `L4_VCON_WRITE_OP`: write bytes to the console.  The kernel does not
/// answer, so the operation only sends and asks to be scheduled.
pub struct Write;

impl Operation for Write {
    type Proto = Protocol;
    const OPCODE: Option<umword> = Some(0);
    const INVOCATION: Invocation = Invocation::Send;
    const TAG_FLAGS: TagFlags = TagFlags::SCHEDULE;
    type Request<'a> = protocol::Bytes<'a>;
    type Reply = ();
}
