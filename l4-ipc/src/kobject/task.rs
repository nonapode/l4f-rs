use crate::protocol;
use l4_ipc_core::handle;

handle!(
    /// A task, the kernel object of an address space with a capability
    /// space.
    pub Task: protocol::Task
);

#[cfg(all(test, not(target_os = "none")))]
mod tests {
    use super::*;
    use crate::extension;
    use l4_ipc_core::CapIdx;

    #[test]
    fn a_task_can_map_and_unmap() {
        fn needs_task<T: extension::Task>(_: T) {}
        needs_task(Task::new(CapIdx::from_index(1)));
    }
}
