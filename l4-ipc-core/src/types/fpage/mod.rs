//! Flexpages: a range of memory, I/O ports or capability slots, used to
//! describe what is mapped or granted by an IPC.

mod order;
mod page;
mod rights;

pub use order::Order;
pub use page::{Page, Type};
pub use rights::{CapRights, MemRights};
