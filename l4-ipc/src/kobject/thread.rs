use crate::protocol;
use l4_ipc_core::handle;

handle!(
    /// A thread, the kernel object that executes code.  It speaks the thread
    /// protocol: its control, register exchange and vCPU operations.
    pub Thread: protocol::Thread
);

#[cfg(all(test, not(target_os = "none")))]
mod tests {
    use super::*;
    use crate::extension;
    use l4_ipc_core::CapIdx;

    #[test]
    fn a_thread_is_a_thread() {
        fn needs_thread<T: extension::Thread>(_: T) {}
        needs_thread(Thread::new(CapIdx::from_index(3)));
    }
}
