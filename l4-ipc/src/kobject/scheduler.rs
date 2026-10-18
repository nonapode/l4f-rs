use crate::protocol;
use l4_ipc_core::handle;

handle!(
    /// A scheduler: the kernel object that manages CPU resources.  It speaks
    /// the scheduler protocol.
    pub Scheduler: protocol::Scheduler
);

#[cfg(all(test, not(target_os = "none")))]
mod tests {
    use super::*;
    use crate::extension;
    use l4_ipc_core::CapIdx;

    #[test]
    fn a_scheduler_is_a_scheduler() {
        fn needs_scheduler<S: extension::Scheduler>(_: S) {}
        needs_scheduler(Scheduler::new(CapIdx::from_index(7)));
    }
}
