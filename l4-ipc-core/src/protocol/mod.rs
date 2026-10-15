//! Protocol support: marshalling of requests and replies and the generic
//! machinery to invoke protocol operations on capability handles.

mod creatable;
mod handle;
mod marshal;
mod operation;
mod vararg;

pub use creatable::Creatable;
pub use handle::{Handle, RawCap, Speaks};
pub use marshal::{Marshal, Reader, Unmarshal, Writer};
pub use operation::{encode_request, Invocation, Operation, Protocol};
pub use vararg::{Int, LStr, Nil, Str, Uint};
