use l4_ipc_core::message::MapItem;
use l4_ipc_core::protocol::{Creatable, Marshal, Operation, Protocol, Uint, Writer};
use l4_ipc_core::sys::{mword, umword};
use l4_ipc_core::types::fpage::{CapRights, Fpage, Object, Order, SndBase};
use l4_ipc_core::{CapIdx, Error};

/// The thread group protocol marker (`L4_PROTO_THREAD_GROUP`).
///
/// A thread group is a collection of threads that IPC gates and IRQs can be
/// bound to, so that the kernel selects one of them as the receiver.  All
/// threads of a group must belong to the same task.
pub struct ThreadGroup;

impl Protocol for ThreadGroup {
    const PROTO: mword = -9;
}

/// How the kernel selects a thread of a thread group as the receiver
/// (`l4_thread_group_policy`).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Policy {
    /// Only use a thread on the same CPU core; fail if there is none
    /// (`L4_THREAD_GROUP_POLICY_STRICT_CORE_LOCAL`).
    StrictCoreLocal,
    /// Prefer a thread on the same CPU core and fall back to any other
    /// (`L4_THREAD_GROUP_POLICY_SOFT_CORE_LOCAL`).
    SoftCoreLocal,
}

impl Policy {
    /// The value of the policy.
    pub const fn bits(self) -> umword {
        match self {
            Policy::StrictCoreLocal => 0,
            Policy::SoftCoreLocal => 1,
        }
    }
}

impl Marshal for Policy {
    fn marshal(&self, w: &mut Writer<'_>) -> Result<(), Error> {
        Uint(self.bits()).marshal(w)
    }
}

/// A factory creates a thread group with a [`Policy`]
/// (`l4_factory_create_thread_group`).
impl Creatable for ThreadGroup {
    type Handle = crate::kobject::ThreadGroup;
    type Args<'a> = Policy;
    type Reply = ();
}

const ADD: umword = 2;
const REMOVE: umword = 3;

/// A thread, as a map item with read, write and strong rights.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ThreadRef(pub CapIdx);

impl Marshal for ThreadRef {
    fn marshal(&self, w: &mut Writer<'_>) -> Result<(), Error> {
        let thread = Fpage::<Object>::new(self.0, Order::ZERO, CapRights::RWS)?;
        MapItem::new(thread, SndBase::default()).marshal(w)
    }
}

/// `l4_thread_group_add`: add the thread to the group.
pub struct Add;

impl Operation for Add {
    type Proto = ThreadGroup;
    const OPCODE: Option<umword> = Some(ADD);
    type Request<'a> = ThreadRef;
    type Reply = ();
}

/// `l4_thread_group_remove`: remove the thread from the group.
pub struct Remove;

impl Operation for Remove {
    type Proto = ThreadGroup;
    const OPCODE: Option<umword> = Some(REMOVE);
    type Request<'a> = ThreadRef;
    type Reply = ();
}

#[cfg(all(test, not(target_os = "none")))]
mod tests {
    use super::*;
    use l4_ipc_core::message::TagFlags;
    use l4_ipc_core::protocol::encode_request;
    use l4_ipc_core::sys::Utcb;

    fn thread(index: usize) -> umword {
        index << 12 | 3 << 4 | 7
    }

    #[test]
    fn protocol_value_matches_the_kernel() {
        assert_eq!(ThreadGroup::PROTO, -9);
    }

    #[test]
    fn policies_match_the_kernel() {
        assert_eq!(Policy::StrictCoreLocal.bits(), 0);
        assert_eq!(Policy::SoftCoreLocal.bits(), 1);
    }

    #[test]
    fn add_and_remove_send_the_thread_as_an_item() {
        for (op, label) in [(ADD, Add::OPCODE), (REMOVE, Remove::OPCODE)] {
            assert_eq!(label, Some(op));
        }

        let utcb = Utcb::zeroed();
        let tag = encode_request::<Add>(&utcb, &ThreadRef(CapIdx::from_index(3))).unwrap();
        assert_eq!(
            (tag.label(), tag.words(), tag.items(), tag.flags()),
            (-9, 1, 1, TagFlags::NONE)
        );
        assert_eq!([utcb.mr(0), utcb.mr(1), utcb.mr(2)], [ADD, 8, thread(3)]);

        let utcb = Utcb::zeroed();
        let tag = encode_request::<Remove>(&utcb, &ThreadRef(CapIdx::from_index(4))).unwrap();
        assert_eq!((tag.words(), tag.items()), (1, 1));
        assert_eq!([utcb.mr(0), utcb.mr(1), utcb.mr(2)], [REMOVE, 8, thread(4)]);
    }

    #[cfg(feature = "proto-factory")]
    #[test]
    fn creating_a_thread_group_sends_the_policy() {
        use crate::protocol::factory::{Create, CreateRequest};

        let utcb = Utcb::zeroed();
        let request = CreateRequest::<ThreadGroup> {
            slot: CapIdx::from_index(0x20),
            args: Policy::SoftCoreLocal,
        };
        let tag = encode_request::<Create<ThreadGroup>>(&utcb, &request).unwrap();

        assert_eq!((tag.label(), tag.words(), tag.items()), (-15, 3, 0));
        assert_eq!(utcb.mr(0), -9isize as umword);
        assert_eq!(utcb.mr(2), 1);
    }
}
