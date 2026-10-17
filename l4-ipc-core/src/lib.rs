//! IPC mechanisms on top of `l4-sys`: the IPC system call, message tags,
//! labels, timeouts and the marshalling of protocol messages into the UTCB.
#![cfg_attr(any(not(test), target_os = "none"), no_std)]

pub use l4_sys as sys;

mod cap;

pub use cap::CapIdx;
mod label;
pub mod message;
mod timeout;
pub mod types;

pub use label::Label;
pub use timeout::Timeout;

mod error;
mod flags;
mod ipc;

pub use error::Error;
pub use ipc::{Dest, IPC};
pub mod protocol;
