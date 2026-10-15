use l4_ipc_core::protocol::{Creatable, Speaks};
use l4_ipc_core::sys::Utcb;
use l4_ipc_core::{CapIdx, Error, Timeout};

use crate::protocol::factory::{Create, CreateRequest, Factory as Protocol};

/// Convenience methods for handles that speak the factory protocol.
///
/// Creating an object returns a handle of the type the object declares
/// through [`Creatable`], so the new capability is usable with the right
/// protocols right away:
///
/// ```no_run
/// use l4_ipc::prelude::*;
/// use l4_ipc::protocol::Factory;
/// use l4_ipc::CapIdx;
/// use l4_ipc_core::handle;
/// use l4_ipc_core::protocol::{Creatable, Protocol};
///
/// struct Object;
/// impl Protocol for Object { const PROTO: isize = -99; }
/// handle!(ObjectHandle: Object);
/// impl Creatable for Object {
///     type Handle = ObjectHandle;
///     type Args<'a> = ();
///     type Reply = ();
/// }
///
/// handle!(Allocator: Factory);
///
/// fn make(allocator: Allocator) -> Result<ObjectHandle, l4_ipc::Error> {
///     allocator.create::<Object>(CapIdx::from_index(0x20), ())
/// }
/// ```
///
/// Only handles that speak the factory protocol can create objects:
///
/// ```compile_fail,E0599
/// use l4_ipc::prelude::*;
/// use l4_ipc::CapIdx;
/// use l4_ipc_core::handle;
/// use l4_ipc_core::protocol::{Creatable, Protocol};
///
/// struct Object;
/// impl Protocol for Object { const PROTO: isize = -99; }
/// handle!(ObjectHandle: Object);
/// impl Creatable for Object {
///     type Handle = ObjectHandle;
///     type Args<'a> = ();
///     type Reply = ();
/// }
///
/// // ObjectHandle does not speak the factory protocol.
/// fn make(not_a_factory: ObjectHandle) {
///     let _ = not_a_factory.create::<Object>(CapIdx::from_index(0x20), ());
/// }
/// ```
pub trait Factory: Speaks<Protocol> {
    /// Create an object of type `T` using the given UTCB.
    ///
    /// The kernel stores the capability of the new object in `slot`, which
    /// has to be a free slot of the capability space; the returned handle
    /// wraps it.  Creating objects that return data besides the capability
    /// (`T::Reply` is not `()`) needs [`Create`] with `invoke_with`.
    fn create_with<T>(
        &self,
        utcb: &Utcb,
        slot: CapIdx,
        args: T::Args<'_>,
    ) -> Result<T::Handle, Error>
    where
        T: Creatable<Reply = ()>,
    {
        self.invoke_with::<Create<T>>(utcb, CreateRequest { slot, args }, Timeout::NEVER)?;
        Ok(T::Handle::from(slot))
    }

    /// Create an object of type `T` using the current UTCB, see
    /// [`create_with`](Factory::create_with).
    fn create<T>(&self, slot: CapIdx, args: T::Args<'_>) -> Result<T::Handle, Error>
    where
        T: Creatable<Reply = ()>,
    {
        self.create_with::<T>(Utcb::current(), slot, args)
    }
}

impl<H: Speaks<Protocol>> Factory for H {}

#[cfg(all(test, not(target_os = "none")))]
mod tests {
    use super::*;
    use l4_ipc_core::protocol::Uint;
    use l4_ipc_core::sys::mword;

    struct Widget;
    impl l4_ipc_core::protocol::Protocol for Widget {
        const PROTO: mword = -99;
    }

    l4_ipc_core::handle!(WidgetHandle: Widget);

    impl Creatable for Widget {
        type Handle = WidgetHandle;
        type Args<'a> = Uint;
        type Reply = ();
    }

    l4_ipc_core::handle!(Allocator: Protocol);

    /// Creating hands back the handle type the object declares.  The
    /// function is only type-checked, creating needs a kernel.
    #[allow(dead_code)]
    fn typed(allocator: Allocator) -> Result<WidgetHandle, Error> {
        allocator.create::<Widget>(CapIdx::from_index(1), Uint(0))
    }

    #[test]
    fn every_handle_that_speaks_the_protocol_is_a_factory() {
        fn needs_factory<F: Factory>(f: F) -> CapIdx {
            f.idx()
        }
        let idx = CapIdx::from_index(2);
        assert_eq!(needs_factory(Allocator::new(idx)), idx);
    }
}
