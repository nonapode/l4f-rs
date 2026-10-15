use super::{Handle, Marshal, Protocol, Unmarshal};
use crate::CapIdx;

/// A kernel object type that a factory can create.
///
/// The implementor is the protocol marker of the object type; its
/// [`PROTO`](Protocol::PROTO) tells the factory which type to create.  The
/// handle type of the new object, and with it the protocols it speaks, is
/// part of the implementation, so creating an object yields a handle that
/// is typed right away.
///
/// The trait is deliberately independent of the factory protocol itself:
/// protocol modules declare what they are created as without depending on
/// the factory being enabled.
pub trait Creatable: Protocol {
    /// The handle for the capability of a new object.
    type Handle: Handle + From<CapIdx>;

    /// The arguments the factory needs besides the object type, for
    /// example a limit.  Use `()` if there are none.
    type Args<'a>: Marshal;

    /// The data the factory returns on success.  Use `()` if there is
    /// none.
    type Reply: Unmarshal;
}

#[cfg(all(test, not(target_os = "none")))]
mod tests {
    use super::*;
    use crate::protocol::LStr;
    use l4_sys::{mword, umword};

    struct Widget;
    impl Protocol for Widget {
        const PROTO: mword = -99;
    }

    crate::handle!(WidgetHandle: Widget);

    impl Creatable for Widget {
        type Handle = WidgetHandle;
        type Args<'a> = LStr<'a>;
        type Reply = (umword,);
    }

    fn wrap<T: Creatable>(slot: CapIdx) -> T::Handle {
        T::Handle::from(slot)
    }

    #[test]
    fn creatable_yields_a_typed_handle() {
        let slot = CapIdx::from_index(9);
        let widget: WidgetHandle = wrap::<Widget>(slot);
        assert_eq!(widget, WidgetHandle::new(slot));
        assert_eq!(widget.idx(), slot);
    }

    #[test]
    fn arguments_may_borrow() {
        fn args(name: &[u8]) -> <Widget as Creatable>::Args<'_> {
            LStr(name)
        }
        assert_eq!(args(b"w").0, b"w");
    }
}
