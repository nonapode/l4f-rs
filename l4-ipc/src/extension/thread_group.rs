use l4_ipc_core::protocol::Speaks;
use l4_ipc_core::sys::Utcb;
use l4_ipc_core::{Error, Timeout};

use crate::protocol::thread::Thread as ThreadProto;
use crate::protocol::thread_group::{Add, Remove, ThreadGroup as Protocol, ThreadRef};

/// Convenience methods for handles that speak the thread group protocol.
///
/// The thread group a method is called on is the one that gains or loses a
/// thread.  Every method has a `_with` variant that takes the UTCB, and
/// takes any handle that speaks the thread protocol.
///
/// ```no_run
/// use l4_ipc::kobject;
/// use l4_ipc::prelude::*;
///
/// fn bind(group: kobject::ThreadGroup, thread: kobject::Thread) -> Result<(), l4_ipc::Error> {
///     group.add(&thread)?;
///     group.remove(&thread)
/// }
/// ```
///
/// Only handles that speak the thread group protocol are thread groups:
///
/// ```compile_fail,E0599
/// use l4_ipc::kobject;
/// use l4_ipc::prelude::*;
/// use l4_ipc_core::handle;
/// use l4_ipc_core::protocol::Protocol;
///
/// struct Other;
/// impl Protocol for Other { const PROTO: isize = -99; }
/// handle!(Thing: Other);
///
/// // Thing is not a thread group.
/// fn bind(thing: Thing, thread: kobject::Thread) {
///     let _ = thing.add(&thread);
/// }
/// ```
pub trait ThreadGroup: Speaks<Protocol> {
    /// Add `thread` to the group (`l4_thread_group_add`), using the given
    /// UTCB.  The thread must be bound to a task and must not already be in
    /// another group.
    fn add_with<T>(&self, utcb: &Utcb, thread: &T) -> Result<(), Error>
    where
        T: Speaks<ThreadProto>,
    {
        self.invoke_with::<Add>(utcb, ThreadRef(thread.idx()), Timeout::NEVER)
    }

    /// See [`add_with`](ThreadGroup::add_with).
    fn add<T>(&self, thread: &T) -> Result<(), Error>
    where
        T: Speaks<ThreadProto>,
    {
        self.add_with(Utcb::current(), thread)
    }

    /// Remove `thread` from the group (`l4_thread_group_remove`), using the
    /// given UTCB.
    fn remove_with<T>(&self, utcb: &Utcb, thread: &T) -> Result<(), Error>
    where
        T: Speaks<ThreadProto>,
    {
        self.invoke_with::<Remove>(utcb, ThreadRef(thread.idx()), Timeout::NEVER)
    }

    /// See [`remove_with`](ThreadGroup::remove_with).
    fn remove<T>(&self, thread: &T) -> Result<(), Error>
    where
        T: Speaks<ThreadProto>,
    {
        self.remove_with(Utcb::current(), thread)
    }
}

impl<H: Speaks<Protocol>> ThreadGroup for H {}

#[cfg(all(test, not(target_os = "none")))]
mod tests {
    use super::*;
    use crate::kobject;
    use l4_ipc_core::handle;
    use l4_ipc_core::CapIdx;

    handle!(Group: Protocol);

    /// The methods are only type-checked, calling them needs a kernel.
    #[allow(dead_code)]
    fn typed(group: Group, thread: kobject::Thread) -> Result<(), Error> {
        group.add(&thread)?;
        group.remove(&thread)
    }

    #[test]
    fn every_handle_that_speaks_the_protocol_is_a_thread_group() {
        fn needs_group<G: ThreadGroup>(g: G) -> CapIdx {
            g.idx()
        }
        let idx = CapIdx::from_index(4);
        assert_eq!(needs_group(Group::new(idx)), idx);
    }
}
