use crate::protocol;
use l4_ipc_core::handle;

handle!(
    /// A thread group: a set of threads that IPC gates and IRQs can be bound
    /// to as a common receiver.  It speaks the thread group protocol.
    pub ThreadGroup: protocol::ThreadGroup
);

#[cfg(all(test, not(target_os = "none")))]
mod tests {
    use super::*;
    use crate::extension;
    use l4_ipc_core::CapIdx;

    #[test]
    fn a_thread_group_is_a_thread_group() {
        fn needs_group<G: extension::ThreadGroup>(_: G) {}
        needs_group(ThreadGroup::new(CapIdx::from_index(4)));
    }
}
