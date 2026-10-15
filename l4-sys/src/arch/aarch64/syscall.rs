//! AArch64 syscall ABI: `svc #0` (or `hvc #0` with the `cpu-virt` feature,
//! matching `CONFIG_KERNEL_CPU_VIRT`).
//!
//! Register assignment, in the generic argument order
//! `(tag, label, dest | flags, timeout)`: x0, x1, x2, x3; the results are
//! returned in x0 (tag) and x1 (label).  This follows the signature of
//! `__l4_sys_syscall` in the L4Re headers.
//!
//! NOTE: not verified by compilation (no aarch64 core available when this
//! was written); clobbers beyond x0-x3 are unknown.

/// Hand the architecture's syscall register list to `$m`:
/// `$m!([ <registers> ] <rest>)`.
macro_rules! register {
    ($m:path, $($rest:tt)*) => {
        $m!([ "x0", "x1", "x2", "x3" ] $($rest)*)
    };
}
pub(crate) use register;

/// Issue the syscall with the given `asm!` operands.
#[cfg(not(feature = "cpu-virt"))]
macro_rules! asm {
    ($($operands:tt)*) => {
        unsafe { core::arch::asm!("svc #0", $($operands)*) }
    };
}

/// Issue the syscall with the given `asm!` operands.
#[cfg(feature = "cpu-virt")]
macro_rules! asm {
    ($($operands:tt)*) => {
        unsafe { core::arch::asm!("hvc #0", $($operands)*) }
    };
}
pub(crate) use asm;
