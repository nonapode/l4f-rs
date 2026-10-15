use l4_ipc_core::protocol::{Operation, Protocol, Reader, Unmarshal};
use l4_ipc_core::sys::{mword, umword};
use l4_ipc_core::Error;

/// The kobject protocol marker (`L4_PROTO_KOBJECT`).
///
/// It is the protocol of the operations that are not specific to a kind of
/// kernel object, and the kernel routes it to IPC gates and IRQs.  Tasks,
/// threads and factories do not speak it: they answer with
/// `-L4_EBADPROTO`.  So no handle of this crate speaks it yet; the handles of
/// IPC gates and IRQs will.
pub struct Kobject;

impl Protocol for Kobject {
    const PROTO: mword = -10;
}

/// The reference count of a capability, the reply of [`DecRefcnt`].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Refcount(pub mword);

impl Unmarshal for Refcount {
    fn unmarshal(r: &mut Reader<'_>) -> Result<Self, Error> {
        r.pop().map(|count| Refcount(count as mword))
    }
}

/// `l4_kobject_dec_refcnt`: decrement the reference count of the capability
/// by the difference, which is the request.  The reply is the new count.  The
/// capability needs the server right (the `S` right of the capability, which
/// the kernel calls the server right of IPC gates).
pub struct DecRefcnt;

impl Operation for DecRefcnt {
    type Proto = Kobject;
    const OPCODE: Option<umword> = Some(0);
    type Request<'a> = mword;
    type Reply = Refcount;
}

#[cfg(all(test, not(target_os = "none")))]
mod tests {
    use super::*;
    use l4_ipc_core::message::{Tag, TagFlags};
    use l4_ipc_core::protocol::encode_request;
    use l4_ipc_core::sys::Utcb;

    #[test]
    fn protocol_value_matches_the_kernel() {
        assert_eq!(Kobject::PROTO, -10);
    }

    #[test]
    fn dec_refcnt_sends_the_difference() {
        let utcb = Utcb::zeroed();
        let tag = encode_request::<DecRefcnt>(&utcb, &-1).unwrap();

        // l4_kobject_dec_refcnt: op 0 and the difference, two words.
        assert_eq!(
            (tag.label(), tag.words(), tag.items(), tag.flags()),
            (-10, 2, 0, TagFlags::NONE)
        );
        assert_eq!((utcb.mr(0), utcb.mr(1)), (0, -1isize as umword));
    }

    #[test]
    fn the_reply_is_the_new_count() {
        let utcb = Utcb::zeroed();
        utcb.set_mr(0, 3);

        let tag = Tag::new(0, 1, 0, TagFlags::NONE);
        let count = Refcount::unmarshal(&mut Reader::new(&utcb, tag));

        assert_eq!(count, Ok(Refcount(3)));
    }

    #[test]
    fn a_reply_without_the_count_is_truncated() {
        let utcb = Utcb::zeroed();
        let tag = Tag::new(0, 0, 0, TagFlags::NONE);

        assert_eq!(
            Refcount::unmarshal(&mut Reader::new(&utcb, tag)),
            Err(Error::Truncated)
        );
    }
}
