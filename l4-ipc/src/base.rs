//! The well-known capabilities of the initial environment.

use super::kobject::{Factory, Scheduler, Task, Thread, Vcon};
use l4_ipc_core::CapIdx;

pub struct Index;

impl Index {
    pub const TASK: CapIdx = CapIdx::from_index(1usize);
    pub const FACTORY: CapIdx = CapIdx::from_index(2usize);
    pub const THREAD: CapIdx = CapIdx::from_index(3usize);
    pub const PAGER: CapIdx = CapIdx::from_index(4usize);
    pub const LOG: CapIdx = CapIdx::from_index(5usize);
    pub const ICU: CapIdx = CapIdx::from_index(6usize);
    pub const SCHEDULER: CapIdx = CapIdx::from_index(7usize);
    pub const IOMMU: CapIdx = CapIdx::from_index(8usize);
    pub const DEBUGGER: CapIdx = CapIdx::from_index(10usize);
    pub const ARM_SMCCC: CapIdx = CapIdx::from_index(11usize);
}

pub const TASK: Task = Task::new(Index::TASK);

pub const THREAD: Thread = Thread::new(Index::THREAD);

pub const LOG: Vcon = Vcon::new(Index::LOG);

pub const FACTORY: Factory = Factory::new(Index::FACTORY);

pub const SCHEDULER: Scheduler = Scheduler::new(Index::SCHEDULER);

#[cfg(all(test, not(target_os = "none")))]
mod tests {
    use super::*;
    use l4_ipc_core::protocol::Handle;

    #[test]
    fn the_factory_capability_is_at_its_well_known_slot() {
        assert_eq!(FACTORY.idx(), CapIdx::from_index(2));
    }

    #[test]
    fn the_thread_capability_is_at_its_well_known_slot() {
        assert_eq!(THREAD.idx(), CapIdx::from_index(3));
    }

    #[test]
    fn the_task_capability_is_at_its_well_known_slot() {
        assert_eq!(TASK.idx(), CapIdx::from_index(1));
    }

    #[test]
    fn the_scheduler_capability_is_at_its_well_known_slot() {
        assert_eq!(SCHEDULER.idx(), CapIdx::from_index(7));
    }
}
