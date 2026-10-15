//! The L4 log protocol (`L4_PROTO_LOG`, implemented by the kernel's
//! virtual console `Vcon` and by log services): write text to a console.
//!
//! ```ignore
//! use l4_ipc_core::{CapIdx, protocol::Cap};
//! use l4_proto_log::{Log, LogExt};
//!
//! let log = Cap::<Log>::new(CapIdx::from_index(LOG_SLOT));
//! log.print("hello rusty world!\n")?;
//! ```
#![cfg_attr(not(test), no_std)]

use core::mem::size_of;
use l4_ipc_core::message::TagFlags;
use l4_ipc_core::protocol::{Cap, Invocation, Marshal, Operation, Protocol, Writer};
use l4_ipc_core::{Error, Timeout};
use l4_sys::{mword, umword, Utcb, MR_COUNT};

/// The log protocol marker (`L4_PROTO_LOG`).
pub struct Log;

impl Protocol for Log {
    const PROTO: mword = -13;
}

/// Maximum number of bytes one write operation transfers
/// (`L4_VCON_WRITE_SIZE`): all message registers except opcode and size.
pub const WRITE_SIZE: usize = (MR_COUNT - 2) * size_of::<umword>();

/// A byte string to write.  Its length must not exceed [`WRITE_SIZE`].
pub struct Bytes<'a>(pub &'a [u8]);

impl Marshal for Bytes<'_> {
    fn marshal(&self, w: &mut Writer<'_>) -> Result<(), Error> {
        w.push(self.0.len() as umword)?;
        w.push_bytes(self.0)
    }
}

/// Operations of the log protocol.
pub mod op {
    use super::*;

    /// `L4_VCON_WRITE_OP`: write bytes to the console.  The kernel does not
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
}

/// Convenience methods for log capabilities.
pub trait LogExt {
    /// Write `bytes` (at most [`WRITE_SIZE`]) using the given UTCB.
    fn write_with(&self, utcb: &Utcb, bytes: &[u8]) -> Result<(), Error>;

    /// Write `text` in as many messages as needed using the given UTCB.
    fn print_with(&self, utcb: &Utcb, text: &str) -> Result<(), Error>;

    /// Write `bytes` (at most [`WRITE_SIZE`]) using the current UTCB.
    fn write(&self, bytes: &[u8]) -> Result<(), Error> {
        self.write_with(Utcb::current(), bytes)
    }

    /// Write `text` in as many messages as needed using the current UTCB.
    fn print(&self, text: &str) -> Result<(), Error> {
        self.print_with(Utcb::current(), text)
    }
}

impl LogExt for Cap<Log> {
    fn write_with(&self, utcb: &Utcb, bytes: &[u8]) -> Result<(), Error> {
        self.invoke_with::<op::Write>(utcb, Bytes(bytes), Timeout::NEVER)
    }

    fn print_with(&self, utcb: &Utcb, text: &str) -> Result<(), Error> {
        for chunk in text.as_bytes().chunks(WRITE_SIZE) {
            self.write_with(utcb, chunk)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use l4_ipc_core::protocol::encode_request;

    #[test]
    fn write_request_matches_vcon_wire_format() {
        let utcb = Utcb::zeroed();
        let tag = encode_request::<op::Write>(&utcb, &Bytes(b"hello")).unwrap();

        assert_eq!(tag.label(), -13);
        assert_eq!(tag.words(), 3); // opcode, size, one word of text
        assert_eq!(tag.items(), 0);
        assert_eq!(tag.flags(), TagFlags::SCHEDULE);

        assert_eq!(utcb.mr(0), 0); // L4_VCON_WRITE_OP
        assert_eq!(utcb.mr(1), 5);
        let mut text = [0u8; size_of::<umword>()];
        text[..5].copy_from_slice(b"hello");
        assert_eq!(utcb.mr(2), umword::from_ne_bytes(text));
    }

    #[test]
    fn maximum_write_fits_exactly() {
        let utcb = Utcb::zeroed();
        let max = [b'x'; WRITE_SIZE];
        let tag = encode_request::<op::Write>(&utcb, &Bytes(&max)).unwrap();
        assert_eq!(tag.words(), MR_COUNT);

        let too_long = [b'x'; WRITE_SIZE + 1];
        assert_eq!(
            encode_request::<op::Write>(&utcb, &Bytes(&too_long)),
            Err(Error::Overflow)
        );
    }
}
