use l4_ipc_core::message::MapItem;
use l4_ipc_core::protocol::{Marshal, Operation, Protocol, Reader, Unmarshal, Writer};
use l4_ipc_core::sys::{mword, umword};
use l4_ipc_core::types::fpage::{CapRights, Fpage, Object, Order, SndBase};
use l4_ipc_core::{CapIdx, Error, Label};

/// The receive endpoint protocol marker, which is `L4_PROTO_KOBJECT`.
///
/// A receive endpoint is a kernel object that is bound to a thread, which
/// receives what is sent to the endpoint.  IPC gates are receive endpoints.
pub struct RcvEndpoint;

impl Protocol for RcvEndpoint {
    const PROTO: mword = -10;
}

/// The label an endpoint is bound with, which the kernel puts into what the
/// receiver gets from the endpoint.
///
/// The two least significant bits are not part of the label: the kernel
/// sets them to the strong and write rights of the capability the sender
/// used, so a receiver can tell the rights apart.  A label with one of these
/// bits set is not accepted by the kernel.  The [`Label`] a receiver gets
/// has these bits set; this is the label as it is configured, so it has
/// them clear.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default, Debug)]
pub struct EndpointLabel(umword);

impl EndpointLabel {
    /// The bits of a label that belong to the rights of the sender.
    pub const RIGHTS: umword = 0b11;

    /// The label, if the bits for the rights are clear.
    pub const fn new(label: umword) -> Option<EndpointLabel> {
        if label & Self::RIGHTS == 0 {
            Some(EndpointLabel(label))
        } else {
            None
        }
    }

    /// The label, if the bits for the rights of the raw label are clear.
    pub const fn from_label(label: Label) -> Option<EndpointLabel> {
        Self::new(label.get())
    }

    pub const fn get(self) -> umword {
        self.0
    }
}

impl From<EndpointLabel> for Label {
    fn from(label: EndpointLabel) -> Label {
        Label::new(label.0)
    }
}

impl Marshal for EndpointLabel {
    fn marshal(&self, w: &mut Writer<'_>) -> Result<(), Error> {
        w.push(self.0)
    }
}

impl Unmarshal for EndpointLabel {
    fn unmarshal(r: &mut Reader<'_>) -> Result<Self, Error> {
        // The kernel keeps a label without the bits for the rights.
        r.pop()
            .map(|label| EndpointLabel(label & !EndpointLabel::RIGHTS))
    }
}

/// `l4_rcv_ep_bind_thread`: bind the endpoint to a receiver, with a label.
///
/// The request carries the label as a word and the receiver as a map item,
/// an object flexpage with read, write and strong rights.  The receiver may
/// be a thread or a thread group.  The capability of the endpoint needs the
/// server right, and the receiver capability the strong right.
pub struct Bind;

/// The request of [`Bind`].
pub struct BindRequest {
    pub label: EndpointLabel,
    pub target: CapIdx,
}

impl Marshal for BindRequest {
    fn marshal(&self, w: &mut Writer<'_>) -> Result<(), Error> {
        let target = Fpage::<Object>::new(self.target, Order::ZERO, CapRights::RWS)?;

        self.label.marshal(w)?;
        MapItem::new(target, SndBase::default()).marshal(w)
    }
}

impl Operation for Bind {
    type Proto = RcvEndpoint;
    const OPCODE: Option<umword> = Some(0x10);
    type Request<'a> = BindRequest;
    type Reply = ();
}

#[cfg(all(test, not(target_os = "none")))]
mod tests {
    use super::*;
    use l4_ipc_core::message::{Tag, TagFlags};
    use l4_ipc_core::protocol::encode_request;
    use l4_ipc_core::sys::Utcb;

    #[test]
    fn protocol_value_matches_the_kernel() {
        assert_eq!(RcvEndpoint::PROTO, -10);
    }

    #[test]
    fn the_bits_for_the_rights_are_not_a_label() {
        assert_eq!(EndpointLabel::new(0b01), None);
        assert_eq!(EndpointLabel::new(0b10), None);
        assert_eq!(EndpointLabel::new(0xff3), None);
        assert_eq!(EndpointLabel::new(0x40), Some(EndpointLabel(0x40)));
        assert_eq!(EndpointLabel::new(0), Some(EndpointLabel::default()));
    }

    #[test]
    fn an_endpoint_label_is_a_label_without_the_bits_for_the_rights() {
        assert_eq!(
            EndpointLabel::from_label(Label::new(0x40)),
            EndpointLabel::new(0x40)
        );
        assert_eq!(EndpointLabel::from_label(Label::new(0x43)), None);
        assert_eq!(
            Label::from(EndpointLabel::new(0x40).unwrap()),
            Label::new(0x40)
        );
    }

    #[test]
    fn bind_sends_the_label_and_the_target_as_an_item() {
        let utcb = Utcb::zeroed();
        let request = BindRequest {
            label: EndpointLabel::new(0x1230).unwrap(),
            target: CapIdx::from_index(3),
        };
        let tag = encode_request::<Bind>(&utcb, &request).unwrap();

        // l4_rcv_ep_bind_thread: op 0x10 and the label as words, then
        // l4_map_obj_control(0, 0) and l4_obj_fpage(target, 0, RWS).
        assert_eq!(
            (tag.label(), tag.words(), tag.items(), tag.flags()),
            (-10, 2, 1, TagFlags::NONE)
        );
        assert_eq!(
            [utcb.mr(0), utcb.mr(1), utcb.mr(2), utcb.mr(3)],
            [0x10, 0x1230, 8, 3 << 12 | 3 << 4 | 7]
        );
    }

    #[test]
    fn bind_needs_a_valid_target() {
        let utcb = Utcb::zeroed();
        let request = BindRequest {
            label: EndpointLabel::default(),
            target: CapIdx::INVALID,
        };

        assert_eq!(
            encode_request::<Bind>(&utcb, &request),
            Err(Error::Fpage(l4_ipc_core::types::fpage::Error::Object(
                l4_ipc_core::types::fpage::object::Error::InvalidCap
            )))
        );
    }

    #[test]
    fn a_label_is_read_without_the_bits_for_the_rights() {
        let utcb = Utcb::zeroed();
        utcb.set_mr(0, 0x1237);

        let tag = Tag::new(0, 1, 0, TagFlags::NONE);
        let label = EndpointLabel::unmarshal(&mut Reader::new(&utcb, tag));

        assert_eq!(label, Ok(EndpointLabel(0x1234)));
    }
}
