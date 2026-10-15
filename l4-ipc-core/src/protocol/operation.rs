use super::{Marshal, Reader, Unmarshal, Writer};
use crate::message::{Tag, TagFlags};
use crate::{CapIdx, Error, Timeout, IPC};
use core::fmt;
use core::marker::PhantomData;
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

/// A capability to an object speaking protocol `P`.
pub struct Cap<P: Protocol> {
    idx: CapIdx,
    _protocol: PhantomData<fn() -> P>,
}

impl<P: Protocol> Clone for Cap<P> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<P: Protocol> Copy for Cap<P> {}

impl<P: Protocol> fmt::Debug for Cap<P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Cap").field(&self.idx).finish()
    }
}

impl<P: Protocol> Cap<P> {
    /// Interpret the capability selector `idx` as speaking protocol `P`.
    pub const fn new(idx: CapIdx) -> Self {
        Cap {
            idx,
            _protocol: PhantomData,
        }
    }

    /// The capability selector.
    pub const fn idx(&self) -> CapIdx {
        self.idx
    }

    /// Perform `Op` using the given UTCB.
    pub fn invoke_with<Op>(
        &self,
        utcb: &Utcb,
        request: Op::Request<'_>,
        timeout: Timeout,
    ) -> Result<Op::Reply, Error>
    where
        Op: Operation<Proto = P>,
    {
        let tag = encode_request::<Op>(utcb, &request)?;
        let reply = match Op::INVOCATION {
            Invocation::Call => IPC::call(self.idx, tag, timeout),
            Invocation::Send => IPC::send(self.idx, tag, timeout),
        };
        if reply.has_error() {
            return Err(Error::Ipc(utcb.error()));
        }
        if Op::INVOCATION == Invocation::Call && reply.label() < 0 {
            return Err(Error::Proto(reply.label()));
        }
        Op::Reply::unmarshal(&mut Reader::new(utcb, reply))
    }

    /// Perform `Op` using the calling thread's UTCB.
    pub fn invoke<Op>(&self, request: Op::Request<'_>, timeout: Timeout) -> Result<Op::Reply, Error>
    where
        Op: Operation<Proto = P>,
    {
        self.invoke_with::<Op>(Utcb::current(), request, timeout)
    }
}

#[cfg(test)]
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

    #[test]
    fn typed_cap_is_copy() {
        let cap = Cap::<Test>::new(CapIdx::from_index(1));
        let copy = cap;
        assert_eq!(cap.idx(), copy.idx());
    }
}
