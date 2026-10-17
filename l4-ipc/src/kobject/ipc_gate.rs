use crate::protocol;
use l4_ipc_core::handle;

handle!(
    /// An IPC gate with the server right: the kernel object that forwards
    /// what is sent to it to the thread it is bound to.
    ///
    /// It is a kobject and a receive endpoint, and has the operations of an
    /// IPC gate.  The capability of a client does not have the server right
    /// and is not an `IpcGate`; it is typed by the protocol of the service.
    pub IpcGate: protocol::Kobject, protocol::RcvEndpoint, protocol::IpcGate
);

#[cfg(all(test, not(target_os = "none")))]
mod tests {
    use super::*;
    use crate::extension;
    use l4_ipc_core::CapIdx;

    #[test]
    fn a_gate_is_a_kobject_an_endpoint_and_a_gate() {
        fn needs<G>(_: G)
        where
            G: extension::Kobject + extension::RcvEndpoint + extension::IpcGate,
        {
        }
        needs(IpcGate::new(CapIdx::from_index(0x20)));
    }
}
