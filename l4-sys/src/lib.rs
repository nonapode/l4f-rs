//! Raw L4 system call layer: syscall invocation, platform word types and
//! access to the current thread's UTCB.
#![cfg_attr(any(not(test), target_os = "none"), no_std)]

#[cfg(not(any(target_pointer_width = "32", target_pointer_width = "64")))]
compile_error!("l4-sys: only 32-bit and 64-bit targets are supported");

#[cfg_attr(
    all(target_pointer_width = "32", target_endian = "little"),
    path = "platform/32-little.rs"
)]
#[cfg_attr(
    all(target_pointer_width = "32", target_endian = "big"),
    path = "platform/32-big.rs"
)]
#[cfg_attr(
    all(target_pointer_width = "64", target_endian = "little"),
    path = "platform/64-little.rs"
)]
#[cfg_attr(
    all(target_pointer_width = "64", target_endian = "big"),
    path = "platform/64-big.rs"
)]
mod platform;

pub use platform::{mword, umword, BIG_ENDIAN};
