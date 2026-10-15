use crate::protocol;
use l4_ipc_core::handle;

handle!(
    /// A factory, the kernel object that creates other kernel objects.
    pub Factory: protocol::Factory
);

#[cfg(all(test, not(target_os = "none")))]
mod tests {
    use super::*;
    use crate::extension;
    use l4_ipc_core::CapIdx;

    #[test]
    fn a_factory_can_create_objects() {
        fn needs_factory<F: extension::Factory>(_: F) {}
        needs_factory(Factory::new(CapIdx::from_index(2)));
    }
}
