use crate::flags::Flags;
use crate::message::Tag;
use crate::{CapIdx, Label, Timeout};
use l4_sys::syscall::{Input, Marker, Syscall};
use l4_sys::umword;

/// Destination register of the IPC syscall: capability selector OR-ed with
/// the syscall flags.  Only constructible inside this crate.
#[derive(Clone, Copy, Debug)]
pub struct Dest(umword);

impl Input for Dest {
    fn encode(&self, _: Marker) -> umword {
        self.0
    }
}

/// The IPC system call (`l4_ipc`).
///
/// Registers, in order: tag, label, destination | flags, timeout.  The
/// kernel returns the tag and the sender's label; the other two registers
/// are not meaningful afterwards.
///
/// The message words themselves are transferred through the UTCB, which
/// the caller has to fill in before and read after the call.
pub struct IPC;

impl Syscall for IPC {
    type Inputs = (Tag, Label, Dest, Timeout);
    type Outputs = (Tag, Label, (), ());
}

impl IPC {
    #[inline(always)]
    fn ipc(dest: CapIdx, flags: Flags, tag: Tag, timeout: Timeout) -> (Tag, Label) {
        let (tag, label, ..) = Self::invoke((
            tag,
            Label::default(),
            Dest(dest.bits() | flags.bits()),
            timeout,
        ));
        (tag, label)
    }

    /// `l4_ipc_call`: send to `dest`, then wait for its reply.
    pub fn call(dest: CapIdx, tag: Tag, timeout: Timeout) -> Tag {
        Self::ipc(dest, Flags::CALL, tag, timeout).0
    }

    /// `l4_ipc_send`: send only.
    pub fn send(dest: CapIdx, tag: Tag, timeout: Timeout) -> Tag {
        Self::ipc(dest, Flags::SEND, tag, timeout).0
    }

    /// `l4_ipc_receive`: closed receive from `src`.
    pub fn receive(src: CapIdx, timeout: Timeout) -> Tag {
        Self::ipc(src, Flags::RECV, Tag::default(), timeout).0
    }

    /// `l4_ipc_wait`: open wait; also returns the sender's label.
    pub fn wait(timeout: Timeout) -> (Tag, Label) {
        Self::ipc(CapIdx::INVALID, Flags::WAIT, Tag::default(), timeout)
    }

    /// `l4_ipc_send_and_wait`.
    pub fn send_and_wait(dest: CapIdx, tag: Tag, timeout: Timeout) -> (Tag, Label) {
        Self::ipc(dest, Flags::SEND_AND_WAIT, tag, timeout)
    }

    /// `l4_ipc_reply_and_wait`: reply through the reply capability, then
    /// open-wait for the next message.
    pub fn reply_and_wait(tag: Tag, timeout: Timeout) -> (Tag, Label) {
        Self::ipc(CapIdx::INVALID, Flags::REPLY_AND_WAIT, tag, timeout)
    }

    /// `l4_ipc_sleep`: receive on the invalid capability, i.e. only the
    /// timeout ends the call.
    pub fn sleep(timeout: Timeout) -> Tag {
        Self::receive(CapIdx::INVALID, timeout)
    }
}
