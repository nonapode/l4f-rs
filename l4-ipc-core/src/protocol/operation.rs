use super::{Marshal, Reader, Unmarshal, Writer};
use crate::message::{Tag, TagFlags};
use crate::{CapIdx, Error, Timeout, IPC};
use l4_sys::{mword, umword, Utcb};

/// An IPC protocol: the label that identifies it in the message tag
/// (`L4_PROTO_*`).
pub trait Protocol {
    /// The protocol label.
    const PROTO: mword;
}

/// How an operation talks to its object.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Invocation {
    /// Send, then wait for the reply (`l4_ipc_call`).
    Call,
    /// Send only (`l4_ipc_send`).
    Send,
}

/// One operation of a [`Protocol`].
pub trait Operation {
    /// The protocol this operation belongs to.
    type Proto: Protocol;

    /// Opcode written to message register 0 before the request, if any.
    const OPCODE: Option<umword>;

    /// Whether to wait for a reply.
    const INVOCATION: Invocation = Invocation::Call;

    /// Flags to set in the request's message tag.
    const TAG_FLAGS: TagFlags = TagFlags::NONE;

    /// The request, written after the opcode.
    type Request<'a>: Marshal;

    /// The reply.  Use `()` for operations without reply data.
    type Reply: Unmarshal;
}

/// Write the request of `Op` into `utcb` and return the tag to send.
pub fn encode_request<Op: Operation>(utcb: &Utcb, request: &Op::Request<'_>) -> Result<Tag, Error> {
    let mut w = Writer::new(utcb);
    if let Some(opcode) = Op::OPCODE {
        w.push(opcode)?;
    }
    request.marshal(&mut w)?;
    Ok(Tag::new(
        <Op::Proto as Protocol>::PROTO,
        w.words(),
        0,
        Op::TAG_FLAGS,
    ))
}

/// Perform `Op` on the capability `idx`: marshal the request, run the IPC
/// and unmarshal the reply.
///
/// A kernel-flagged IPC error becomes `Error::Ipc` (with the UTCB's error
/// code), a negative reply label of a call `Error::Proto`.
pub(super) fn invoke_raw<Op: Operation>(
    idx: CapIdx,
    utcb: &Utcb,
    request: Op::Request<'_>,
    timeout: Timeout,
) -> Result<Op::Reply, Error> {
    let tag = encode_request::<Op>(utcb, &request)?;
    let reply = match Op::INVOCATION {
        Invocation::Call => IPC::call(idx, tag, timeout),
        Invocation::Send => IPC::send(idx, tag, timeout),
    };
    if reply.has_error() {
        return Err(Error::Ipc(utcb.error()));
    }
    if Op::INVOCATION == Invocation::Call && reply.label() < 0 {
        return Err(Error::Proto(reply.label()));
    }
    Op::Reply::unmarshal(&mut Reader::new(utcb, reply))
}

#[cfg(all(test, not(target_os = "none")))]
mod tests {
    use super::*;

    struct Test;
    impl Protocol for Test {
        const PROTO: mword = -42;
    }

    struct Add;
    impl Operation for Add {
        type Proto = Test;
        const OPCODE: Option<umword> = Some(7);
        const TAG_FLAGS: TagFlags = TagFlags::SCHEDULE;
        type Request<'a> = (umword, umword);
        type Reply = (umword,);
    }

    #[test]
    fn request_layout() {
        let utcb = Utcb::zeroed();
        let tag = encode_request::<Add>(&utcb, &(3, 4)).unwrap();
        assert_eq!(tag.label(), -42);
        assert_eq!(tag.words(), 3);
        assert_eq!(tag.items(), 0);
        assert_eq!(tag.flags(), TagFlags::SCHEDULE);
        assert_eq!((utcb.mr(0), utcb.mr(1), utcb.mr(2)), (7, 3, 4));
    }

    crate::handle!(TestHandle: Test);

    #[test]
    fn handle_declares_its_protocols() {
        use crate::protocol::{RawCap, Speaks};
        fn needs<H: Speaks<Test>>(h: H) -> CapIdx {
            h.idx()
        }
        assert_eq!(
            needs(TestHandle::new(CapIdx::from_index(2))),
            CapIdx::from_index(2)
        );
        assert_eq!(
            needs(RawCap::new(CapIdx::from_index(3))),
            CapIdx::from_index(3)
        );
    }
}
