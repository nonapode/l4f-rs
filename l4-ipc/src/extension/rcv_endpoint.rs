use l4_ipc_core::protocol::Speaks;
use l4_ipc_core::sys::Utcb;
use l4_ipc_core::{Error, Timeout};

use crate::protocol::rcv_endpoint::{Bind, BindRequest, EndpointLabel, RcvEndpoint as Protocol};
use crate::protocol::thread::Thread;

/// Convenience methods for handles that speak the receive endpoint protocol.
///
/// ```no_run
/// use l4_ipc::kobject;
/// use l4_ipc::prelude::*;
/// use l4_ipc::protocol::rcv_endpoint::EndpointLabel;
/// use l4_ipc::protocol::RcvEndpoint;
/// use l4_ipc_core::handle;
///
/// handle!(Endpoint: RcvEndpoint);
///
/// fn serve(endpoint: Endpoint, thread: kobject::Thread) -> Result<(), l4_ipc::Error> {
///     endpoint.bind_thread(&thread, EndpointLabel::new(0x1000).unwrap())
/// }
/// ```
///
/// Only handles that speak the receive endpoint protocol can be bound:
///
/// ```compile_fail,E0599
/// use l4_ipc::kobject;
/// use l4_ipc::prelude::*;
/// use l4_ipc::protocol::rcv_endpoint::EndpointLabel;
/// use l4_ipc_core::handle;
/// use l4_ipc_core::protocol::Protocol;
///
/// struct Other;
/// impl Protocol for Other { const PROTO: isize = -99; }
/// handle!(Thing: Other);
///
/// // Thing is not a receive endpoint.
/// fn serve(thing: Thing, thread: kobject::Thread) {
///     let _ = thing.bind_thread(&thread, EndpointLabel::default());
/// }
/// ```
pub trait RcvEndpoint: Speaks<Protocol> {
    /// Bind the endpoint to the thread with the label
    /// (`l4_rcv_ep_bind_thread`), using the given UTCB.
    ///
    /// The endpoint is bound anew if it was bound before.  Messages that
    /// are already in flight keep the old label.
    fn bind_thread_with<T>(
        &self,
        utcb: &Utcb,
        thread: &T,
        label: EndpointLabel,
    ) -> Result<(), Error>
    where
        T: Speaks<Thread>,
    {
        let request = BindRequest {
            label,
            thread: thread.idx(),
        };
        self.invoke_with::<Bind>(utcb, request, Timeout::NEVER)
    }

    /// See [`bind_thread_with`](RcvEndpoint::bind_thread_with).
    fn bind_thread<T>(&self, thread: &T, label: EndpointLabel) -> Result<(), Error>
    where
        T: Speaks<Thread>,
    {
        self.bind_thread_with(Utcb::current(), thread, label)
    }
}

impl<H: Speaks<Protocol>> RcvEndpoint for H {}

#[cfg(all(test, not(target_os = "none")))]
mod tests {
    use super::*;
    use crate::kobject;
    use l4_ipc_core::handle;
    use l4_ipc_core::CapIdx;

    handle!(Endpoint: Protocol);

    /// The methods are only type-checked, calling them needs a kernel.
    #[allow(dead_code)]
    fn typed(endpoint: Endpoint, thread: kobject::Thread) -> Result<(), Error> {
        endpoint.bind_thread(&thread, EndpointLabel::default())
    }

    #[test]
    fn every_handle_that_speaks_the_protocol_is_an_endpoint() {
        fn needs_endpoint<E: RcvEndpoint>(e: E) -> CapIdx {
            e.idx()
        }
        let idx = CapIdx::from_index(4);
        assert_eq!(needs_endpoint(Endpoint::new(idx)), idx);
    }
}
