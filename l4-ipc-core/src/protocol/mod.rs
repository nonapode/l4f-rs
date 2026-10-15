//! Protocol support: marshalling of requests and replies and the generic
//! machinery to invoke protocol operations on capabilities.

mod handle;
mod marshal;
mod operation;

pub use handle::{Handle, RawCap, Speaks};
pub use marshal::{Marshal, Reader, Unmarshal, Writer};
pub use operation::{encode_request, Cap, Invocation, Operation, Protocol};
