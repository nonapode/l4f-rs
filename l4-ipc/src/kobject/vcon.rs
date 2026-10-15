use crate::protocol;
use l4_ipc_core::handle;

handle!(
    /// A virtual console, the kernel object behind the log capability.
    /// Further protocols are added here as they are implemented.
    pub Vcon: protocol::Log
);

#[cfg(all(test, not(target_os = "none")))]
mod tests {
    use super::*;
    use crate::extension::Log;
    use l4_ipc_core::CapIdx;

    #[test]
    fn a_vcon_can_log() {
        fn needs_log<L: Log>(_: L) {}
        needs_log(Vcon::new(CapIdx::from_index(5)));
    }
}
