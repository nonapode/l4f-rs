use core::marker::PhantomData;
use l4_ipc_core::message::RcvCap;
use l4_ipc_core::protocol::{Creatable, Marshal, Operation, Protocol, Uint, Writer};
use l4_ipc_core::sys::{mword, umword, Utcb};
use l4_ipc_core::{CapIdx, Error};

/// The factory protocol marker (`L4_PROTO_FACTORY`).
///
/// A factory creates kernel objects.  What it creates is selected by the
/// protocol of the new object, see [`Creatable`] and [`Create`].
pub struct Factory;

impl Protocol for Factory {
    const PROTO: mword = -15;
}

/// The limit of a factory that is created by a factory: how much kernel
/// memory it may hand out (`l4_factory_create_factory`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Limit(pub umword);

impl Marshal for Limit {
    fn marshal(&self, w: &mut Writer<'_>) -> Result<(), Error> {
        Uint(self.0).marshal(w)
    }
}

/// The request of [`Create`]: the capability slot that receives the new
/// object, and the arguments of the object type.
pub struct CreateRequest<'a, T: Creatable> {
    /// The slot the kernel stores the new capability in.
    pub slot: CapIdx,
    /// The arguments for the new object.
    pub args: T::Args<'a>,
}

impl<T: Creatable> Marshal for CreateRequest<'_, T> {
    fn marshal(&self, w: &mut Writer<'_>) -> Result<(), Error> {
        // Unlike for other protocols the first word is not an opcode but the
        // protocol of the object to create.
        w.push(T::PROTO as umword)?;
        self.args.marshal(w)
    }
}

/// `l4_factory_create`: create an object of type `T`.
///
/// The request carries the protocol of `T` in message register 0, followed
/// by the arguments of `T` as variable arguments.  The kernel hands the
/// capability of the new object back through a receive item for
/// [`CreateRequest::slot`] in the first buffer register.  The factory
/// capability needs the send (`S`) right.  Errors such as `-L4_EPERM` or
/// `-L4_ENOMEM` arrive as [`Error::Proto`].
pub struct Create<T>(PhantomData<fn() -> T>);

impl<T: Creatable> Operation for Create<T> {
    type Proto = Factory;
    const OPCODE: Option<umword> = None;
    type Request<'a> = CreateRequest<'a, T>;
    type Reply = T::Reply;

    fn prepare_receive(utcb: &Utcb, request: &CreateRequest<'_, T>) {
        utcb.set_bdr(0);
        utcb.set_br(0, RcvCap::new(request.slot).bits());
    }

    fn release_receive(utcb: &Utcb) {
        utcb.set_br(0, 0);
    }
}

#[cfg(all(test, not(target_os = "none")))]
mod tests {
    use super::*;
    use core::mem::size_of;
    use l4_ipc_core::message::TagFlags;
    use l4_ipc_core::protocol::{encode_request, LStr};

    struct Widget;
    impl Protocol for Widget {
        const PROTO: mword = -99;
    }

    l4_ipc_core::handle!(WidgetHandle: Widget);

    impl Creatable for Widget {
        type Handle = WidgetHandle;
        type Args<'a> = (Uint, LStr<'a>);
        type Reply = ();
    }

    struct Plain;
    impl Protocol for Plain {
        const PROTO: mword = -98;
    }

    l4_ipc_core::handle!(PlainHandle: Plain);

    impl Creatable for Plain {
        type Handle = PlainHandle;
        type Args<'a> = ();
        type Reply = ();
    }

    const SLOT: CapIdx = CapIdx::from_index(0x20);

    #[test]
    fn protocol_value_matches_the_kernel() {
        assert_eq!(Factory::PROTO, -15);
    }

    #[test]
    fn request_names_the_type_and_appends_the_arguments() {
        let utcb = Utcb::zeroed();
        let request = CreateRequest::<Widget> {
            slot: SLOT,
            args: (Uint(7), LStr(b"w")),
        };
        let tag = encode_request::<Create<Widget>>(&utcb, &request).unwrap();

        assert_eq!(tag.label(), -15);
        assert_eq!(tag.words(), 1 + 2 + 2); // type, uint argument, string argument
        assert_eq!(tag.items(), 0);
        assert_eq!(tag.flags(), TagFlags::NONE);

        assert_eq!(utcb.mr(0), -99isize as umword); // the type, not an opcode
        assert_eq!(utcb.mr(2), 7);
    }

    #[test]
    fn without_arguments_the_request_is_just_the_type() {
        let utcb = Utcb::zeroed();
        let request = CreateRequest::<Plain> {
            slot: SLOT,
            args: (),
        };
        let tag = encode_request::<Create<Plain>>(&utcb, &request).unwrap();
        assert_eq!(tag.words(), 1);
        assert_eq!(utcb.mr(0), -98isize as umword);
    }

    #[test]
    fn limit_is_an_unsigned_word_argument() {
        let utcb = Utcb::zeroed();
        let mut w = Writer::new(&utcb);
        Limit(0x1000).marshal(&mut w).unwrap();
        assert_eq!(w.words(), 2);
        assert_eq!(utcb.mr(0), 0x01 | (size_of::<umword>() as umword) << 16);
        assert_eq!(utcb.mr(1), 0x1000);
    }

    #[test]
    fn the_new_capability_is_received_into_the_slot() {
        let utcb = Utcb::zeroed();
        utcb.set_bdr(0xff); // a stale buffer descriptor must not survive
        let request = CreateRequest::<Plain> {
            slot: SLOT,
            args: (),
        };

        <Create<Plain> as Operation>::prepare_receive(&utcb, &request);
        assert_eq!(utcb.bdr(), 0);
        assert_eq!(utcb.br(0), RcvCap::new(SLOT).bits());

        <Create<Plain> as Operation>::release_receive(&utcb);
        assert_eq!(utcb.br(0), 0);
    }
}
