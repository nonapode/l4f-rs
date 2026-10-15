use crate::protocol::Log;
use l4_ipc_core::protocol::Cap;
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

pub const LOG: Cap<Log> = Cap::<Log>::new(Index::LOG);
