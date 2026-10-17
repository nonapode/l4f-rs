use super::rcv_endpoint::EndpointLabel;
use l4_ipc_core::protocol::{Operation, Protocol};
use l4_ipc_core::sys::{mword, umword};

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
    use l4_ipc_core::message::{Tag, TagFlags};
    use l4_ipc_core::protocol::{encode_request, Reader, Unmarshal};
    use l4_ipc_core::sys::Utcb;

    #[test]
    fn protocol_value_matches_the_kernel() {
        assert_eq!(IpcGate::PROTO, -10);
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
