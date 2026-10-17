use super::rcv_endpoint::EndpointLabel;
use super::thread::Thread;
use l4_ipc_core::message::MapItem;
use l4_ipc_core::protocol::{Creatable, Marshal, Operation, Protocol, Speaks, Uint, Writer};
use l4_ipc_core::sys::{mword, umword};
use l4_ipc_core::types::fpage::{CapRights, Fpage, Obj, Order, SndBase};
use l4_ipc_core::{CapIdx, Error};

/// The IPC gate protocol marker, which is `L4_PROTO_KOBJECT`.
///
/// It is the part of the kernel interface of an IPC gate that is not shared
/// with other objects: the gate is also a kobject and a receive endpoint.
///
/// The operations are only processed by the kernel if the capability has the
/// server right (`L4_FPAGE_C_IPCGATE_SVR`, [`ObjOptions::IPC_GATE_SERVER`]).
/// A capability without it is the client side of the gate: what is invoked
/// on it is forwarded to the thread the gate is bound to, with the label of
/// the gate.  The handle of a client capability therefore speaks the
/// protocol of the service, and not this one.
///
/// [`ObjOptions::IPC_GATE_SERVER`]: l4_ipc_core::types::fpage::ObjOptions::IPC_GATE_SERVER
pub struct IpcGate;

impl Protocol for IpcGate {
    const PROTO: mword = -10;
}

/// The marker for creating an IPC gate with a factory.
///
/// The factory is told the protocol of the object to create, and an IPC gate
/// has none of its own: what is sent through it speaks the protocol of the
/// server behind it.  The type that is sent is therefore `L4_PROTO_NONE`, 0.
pub struct Gate;

impl Protocol for Gate {
    const PROTO: mword = 0;
}

/// What a new IPC gate is bound to: its label and, optionally, the thread.
///
/// The request is the label as a variable argument and, if there is a thread,
/// the thread as a map item (`l4_factory_create_gate`).  A gate without a
/// thread is bound later with [`bind_thread`].
///
/// ```no_run
/// # #[cfg(feature = "proto-factory")]
/// # fn create() -> Result<(), l4_ipc::Error> {
/// use l4_ipc::prelude::*;
/// use l4_ipc::protocol::ipc_gate::{Gate, GateTarget};
/// use l4_ipc::protocol::rcv_endpoint::EndpointLabel;
/// use l4_ipc::{kobject, CapIdx};
///
/// let factory = kobject::Factory::new(CapIdx::from_index(2));
/// let thread = kobject::Thread::new(CapIdx::from_index(3));
/// let label = EndpointLabel::new(0x1000).unwrap();
///
/// let gate: kobject::IpcGate =
///     factory.create::<Gate>(CapIdx::from_index(0x20), GateTarget::bound(label, &thread))?;
/// # Ok(())
/// # }
/// ```
///
/// [`bind_thread`]: crate::extension::RcvEndpoint::bind_thread
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct GateTarget {
    label: EndpointLabel,
    thread: Option<CapIdx>,
}

impl GateTarget {
    /// A gate that is bound to nothing yet.
    pub const fn unbound(label: EndpointLabel) -> GateTarget {
        GateTarget {
            label,
            thread: None,
        }
    }

    /// A gate that is bound to the thread.
    pub fn bound<T: Speaks<Thread>>(label: EndpointLabel, thread: &T) -> GateTarget {
        GateTarget {
            label,
            thread: Some(thread.idx()),
        }
    }

    pub const fn label(&self) -> EndpointLabel {
        self.label
    }

    /// The selector of the thread, if the gate is bound to one.
    pub const fn thread(&self) -> Option<CapIdx> {
        self.thread
    }
}

impl Marshal for GateTarget {
    fn marshal(&self, w: &mut Writer<'_>) -> Result<(), Error> {
        Uint(self.label.get()).marshal(w)?;

        if let Some(thread) = self.thread {
            let thread = Fpage::<Obj>::new(thread, Order::ZERO, CapRights::RWS)?;
            MapItem::new(thread, SndBase::default()).marshal(w)?;
        }
        Ok(())
    }
}

/// A factory creates an IPC gate with the server right, bound as the target
/// says.
impl Creatable for Gate {
    type Handle = crate::kobject::IpcGate;
    type Args<'a> = GateTarget;
    type Reply = ();
}

/// `l4_ipc_gate_get_infos`: the label of the gate.  The reply is the label
/// the gate was bound with, without the bits for the rights of a sender.
pub struct GetInfos;

impl Operation for GetInfos {
    type Proto = IpcGate;
    const OPCODE: Option<umword> = Some(0x11);
    type Request<'a> = ();
    type Reply = EndpointLabel;
}

#[cfg(all(test, not(target_os = "none")))]
mod tests {
    use super::*;
    use core::mem::size_of;
    use l4_ipc_core::message::{Tag, TagFlags};
    use l4_ipc_core::protocol::{encode_request, Reader, Unmarshal};
    use l4_ipc_core::sys::Utcb;

    #[test]
    fn protocol_value_matches_the_kernel() {
        assert_eq!(IpcGate::PROTO, -10);
    }

    fn thread() -> CapIdx {
        CapIdx::from_index(3)
    }

    #[test]
    fn creating_a_gate_has_no_protocol() {
        assert_eq!(Gate::PROTO, 0);
    }

    #[test]
    fn a_bound_target_sends_the_label_and_the_thread() {
        let utcb = Utcb::zeroed();
        let target = GateTarget::bound(
            EndpointLabel::new(0x1230).unwrap(),
            &crate::kobject::Thread::new(thread()),
        );
        let mut w = Writer::new(&utcb);
        target.marshal(&mut w).unwrap();

        // l4_factory_create_gate: the label as a variable argument, then the
        // thread as a map item with l4_map_obj_control(0, 0) and
        // l4_obj_fpage(thread, 0, RWS).
        assert_eq!((w.words(), w.items()), (2, 1));
        assert_eq!(
            [utcb.mr(0), utcb.mr(1), utcb.mr(2), utcb.mr(3)],
            [
                0x1 | (size_of::<umword>() as umword) << 16,
                0x1230,
                8,
                3 << 12 | 3 << 4 | 7
            ]
        );
    }

    #[test]
    fn an_unbound_target_sends_the_label_only() {
        let utcb = Utcb::zeroed();
        let mut w = Writer::new(&utcb);
        GateTarget::unbound(EndpointLabel::new(0x40).unwrap())
            .marshal(&mut w)
            .unwrap();

        assert_eq!((w.words(), w.items()), (2, 0));
        assert_eq!(utcb.mr(1), 0x40);
        assert_eq!(utcb.mr(2), 0);
    }

    #[test]
    fn a_target_needs_a_valid_thread() {
        let utcb = Utcb::zeroed();
        let target = GateTarget {
            label: EndpointLabel::default(),
            thread: Some(CapIdx::INVALID),
        };

        assert_eq!(
            target.marshal(&mut Writer::new(&utcb)),
            Err(Error::Fpage(l4_ipc_core::types::fpage::Error::InvalidCap))
        );
    }

    #[cfg(feature = "proto-factory")]
    #[test]
    fn creating_a_gate_is_a_create_request_with_an_item() {
        use crate::protocol::factory::{Create, CreateRequest};

        let utcb = Utcb::zeroed();
        let request = CreateRequest::<Gate> {
            slot: CapIdx::from_index(0x20),
            args: GateTarget::bound(
                EndpointLabel::new(0x1230).unwrap(),
                &crate::kobject::Thread::new(thread()),
            ),
        };
        let tag = encode_request::<Create<Gate>>(&utcb, &request).unwrap();

        // The type 0, the label argument (two words) and one item.
        assert_eq!((tag.label(), tag.words(), tag.items()), (-15, 3, 1));
        assert_eq!(utcb.mr(0), 0);
        assert_eq!(utcb.mr(2), 0x1230);
        assert_eq!(utcb.mr(3), 8);
    }

    #[test]
    fn get_infos_is_just_the_operation_code() {
        let utcb = Utcb::zeroed();
        let tag = encode_request::<GetInfos>(&utcb, &()).unwrap();

        // l4_ipc_gate_get_infos: L4_IPC_GATE_GET_INFO_OP, one word.
        assert_eq!(
            (tag.label(), tag.words(), tag.items(), tag.flags()),
            (-10, 1, 0, TagFlags::NONE)
        );
        assert_eq!(utcb.mr(0), 0x11);
    }

    #[test]
    fn the_reply_is_the_label() {
        let utcb = Utcb::zeroed();
        utcb.set_mr(0, 0x8000);

        let tag = Tag::new(0, 1, 0, TagFlags::NONE);
        let label = <GetInfos as Operation>::Reply::unmarshal(&mut Reader::new(&utcb, tag));

        assert_eq!(label, Ok(EndpointLabel::new(0x8000).unwrap()));
    }
}
