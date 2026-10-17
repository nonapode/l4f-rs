use l4_ipc_core::protocol::Speaks;

use crate::protocol::thread::Thread as Protocol;

/// Convenience methods for handles that speak the thread protocol.
///
/// There are none yet; the operations of threads are not implemented.  The
/// trait is what a function takes that needs a thread.
pub trait Thread: Speaks<Protocol> {}

impl<H: Speaks<Protocol>> Thread for H {}
