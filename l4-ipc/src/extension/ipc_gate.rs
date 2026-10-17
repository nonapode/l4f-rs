use l4_ipc_core::protocol::Speaks;
use l4_ipc_core::sys::Utcb;
use l4_ipc_core::{Error, Timeout};

use crate::protocol::ipc_gate::{GetInfos, IpcGate as Protocol};
use crate::protocol::rcv_endpoint::EndpointLabel;

/// Convenience methods for handles that speak the IPC gate protocol, i.e.
/// capabilities of an IPC gate that have the server right.
///
/// ```no_run
/// use l4_ipc::kobject::IpcGate;
/// use l4_ipc::prelude::*;
///
/// fn label(gate: IpcGate) -> Result<usize, l4_ipc::Error> {
///     Ok(gate.get_infos()?.get())
/// }
/// ```
///
/// A capability that is only typed by the service behind the gate cannot ask
/// for the label:
///
/// ```compile_fail,E0599
/// use l4_ipc::prelude::*;
/// use l4_ipc_core::handle;
/// use l4_ipc_core::protocol::Protocol;
///
/// struct Service;
/// impl Protocol for Service { const PROTO: isize = 0x1234; }
/// handle!(Client: Service);
///
/// // The kernel would forward this to the server.
/// fn ask(client: Client) {
///     let _ = client.get_infos();
/// }
/// ```
pub trait IpcGate: Speaks<Protocol> {
    /// The label of the gate (`l4_ipc_gate_get_infos`), using the given
    /// UTCB.
    fn get_infos_with(&self, utcb: &Utcb) -> Result<EndpointLabel, Error> {
        self.invoke_with::<GetInfos>(utcb, (), Timeout::NEVER)
    }

    /// See [`get_infos_with`](IpcGate::get_infos_with).
    fn get_infos(&self) -> Result<EndpointLabel, Error> {
        self.get_infos_with(Utcb::current())
    }
}

impl<H: Speaks<Protocol>> IpcGate for H {}
