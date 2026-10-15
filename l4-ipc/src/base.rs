use super::protocol;
use l4_ipc_core::protocol::Creatable;
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
    /// Further protocols are added here as they are implemented.
    pub Vcon: protocol::Log
);

pub const LOG: Vcon = Vcon::new(Index::LOG);

handle!(
    /// A factory, the kernel object that creates other kernel objects.
    pub Factory: protocol::Factory
);

pub const FACTORY: Factory = Factory::new(Index::FACTORY);

/// A factory can create factories; the limit bounds the kernel memory the
/// new factory may hand out.
impl Creatable for protocol::Factory {
    type Handle = Factory;
    type Args<'a> = protocol::factory::Limit;
    type Reply = ();
}

#[cfg(all(test, not(target_os = "none")))]
mod tests {
    use super::*;
    use l4_ipc_core::protocol::{encode_request, Handle};
    use l4_ipc_core::sys::{umword, Utcb};

    #[test]
    fn the_factory_capability_is_at_its_well_known_slot() {
        assert_eq!(FACTORY.idx(), CapIdx::from_index(2));
    }

    #[test]
    fn creating_a_factory_sends_the_limit() {
        let utcb = Utcb::zeroed();
        let request = protocol::factory::CreateRequest::<protocol::Factory> {
            slot: CapIdx::from_index(0x20),
            args: protocol::factory::Limit(0x1000),
        };
        let tag = encode_request::<protocol::factory::Create<protocol::Factory>>(&utcb, &request)
            .unwrap();
        assert_eq!((tag.label(), tag.words(), tag.items()), (-15, 3, 0));
        assert_eq!(utcb.mr(0), -15isize as umword); // the created type is a factory
        assert_eq!(utcb.mr(2), 0x1000);
    }
}
