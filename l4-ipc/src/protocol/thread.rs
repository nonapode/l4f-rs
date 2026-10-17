use l4_ipc_core::protocol::Protocol;
use l4_ipc_core::sys::mword;

/// The thread protocol marker (`L4_PROTO_THREAD`).
///
/// A thread is the unit of execution.  Only the marker is here so far, so
/// that a thread can be named where another protocol takes one, for example
/// as the receiver an IPC gate is bound to.  The operations of the thread
/// protocol are not implemented yet.
pub struct Thread;

impl Protocol for Thread {
    const PROTO: mword = -12;
}

#[cfg(all(test, not(target_os = "none")))]
mod tests {
    use super::*;

    #[test]
    fn protocol_value_matches_the_kernel() {
        assert_eq!(Thread::PROTO, -12);
    }
}
