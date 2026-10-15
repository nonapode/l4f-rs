use l4_ipc_core::protocol::Speaks;
use l4_ipc_core::sys::{mword, Utcb};
use l4_ipc_core::{Error, Timeout};

use crate::protocol::kobject::{DecRefcnt, Kobject as Protocol};

/// Convenience methods for handles that speak the kobject protocol.
///
/// ```no_run
/// use l4_ipc::prelude::*;
/// use l4_ipc::protocol::Kobject;
/// use l4_ipc_core::handle;
///
/// handle!(Gate: Kobject);
///
/// fn release(gate: Gate) -> Result<isize, l4_ipc::Error> {
///     gate.dec_refcnt(1)
/// }
/// ```
///
/// Only handles that speak the kobject protocol can:
///
/// ```compile_fail,E0599
/// use l4_ipc::prelude::*;
/// use l4_ipc_core::handle;
/// use l4_ipc_core::protocol::Protocol;
///
/// struct Other;
/// impl Protocol for Other { const PROTO: isize = -99; }
/// handle!(Thing: Other);
///
/// // Thing does not speak the kobject protocol.
/// fn release(thing: Thing) {
///     let _ = thing.dec_refcnt(1);
/// }
/// ```
pub trait Kobject: Speaks<Protocol> {
    /// Decrement the reference count of the capability by `diff` and return
    /// the new count (`l4_kobject_dec_refcnt`), using the given UTCB.
    fn dec_refcnt_with(&self, utcb: &Utcb, diff: mword) -> Result<mword, Error> {
        let count = self.invoke_with::<DecRefcnt>(utcb, diff, Timeout::NEVER)?;
        Ok(count.0)
    }

    /// See [`dec_refcnt_with`](Kobject::dec_refcnt_with).
    fn dec_refcnt(&self, diff: mword) -> Result<mword, Error> {
        self.dec_refcnt_with(Utcb::current(), diff)
    }
}

impl<H: Speaks<Protocol>> Kobject for H {}

#[cfg(all(test, not(target_os = "none")))]
mod tests {
    use super::*;
    use l4_ipc_core::handle;
    use l4_ipc_core::CapIdx;

    handle!(Gate: Protocol);

    #[test]
    fn every_handle_that_speaks_the_protocol_has_a_refcount() {
        fn needs_kobject<K: Kobject>(k: K) -> CapIdx {
            k.idx()
        }
        let idx = CapIdx::from_index(6);
        assert_eq!(needs_kobject(Gate::new(idx)), idx);
    }
}
