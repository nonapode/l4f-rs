use super::protocol;
use l4_ipc_core::{handle, CapIdx};

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

handle!(
    /// A virtual console, the kernel object behind the log capability.
    /// Further protocols are added here as their crates appear.
    pub Vcon: protocol::Log
);

pub const LOG: Vcon = Vcon::new(Index::LOG);
