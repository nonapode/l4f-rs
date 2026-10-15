//! Raw L4 system call layer: syscall invocation, platform word types and
//! access to the current thread's UTCB.
#![cfg_attr(any(not(test), target_os = "none"), no_std)]

#[cfg(not(any(target_pointer_width = "32", target_pointer_width = "64")))]
compile_error!("l4-sys: only 32-bit and 64-bit targets are supported");

mod platform;

pub use platform::{mword, umword};

#[cfg(not(any(
    target_arch = "x86_64",
    target_arch = "aarch64",
    target_arch = "arm",
    target_arch = "x86"
)))]
compile_error!("l4-sys: unsupported target architecture");

#[cfg_attr(target_arch = "aarch64", path = "arch/aarch64/mod.rs")]
#[cfg_attr(target_arch = "x86_64", path = "arch/x86-64/mod.rs")]
#[cfg_attr(target_arch = "arm", path = "arch/arm/mod.rs")]
#[cfg_attr(target_arch = "x86", path = "arch/x86/mod.rs")]
mod arch;

pub mod syscall;

mod utcb;

pub use utcb::{BR_COUNT, MR_COUNT, TLS_COUNT, Utcb};
