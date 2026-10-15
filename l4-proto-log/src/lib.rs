//! The L4 log protocol (`L4_PROTO_LOG`, implemented by the kernel's
//! virtual console `Vcon` and by log services): write text to a console.
//!
//! Every handle that [`Speaks<Protocol>`](l4_ipc_core::protocol::Speaks) gets the
//! methods of [`Extension`]; [`extension::Console`] adds buffered `core::fmt::Write`
//! output on top of any such handle.
//!
//! ```ignore
//! use core::fmt::Write;
//! use l4_proto_log::Extension;
//!
//! log.print("hello rusty world!\n")?;
//! write!(log.console(), "pid {} started\n", pid)?;
//! ```
#![cfg_attr(not(test), no_std)]

pub mod extension;
pub use extension::Log as Extension;
pub mod protocol;
pub use protocol::Log as Protocol;
pub mod operation;
