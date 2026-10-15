//! Locating the current thread's UTCB on x86-64.

use crate::Utcb;

/// The UTCB pointer is stored at offset 0 of the `gs` segment
/// (`l4_utcb_direct`: `mov %gs:0, reg`).
#[inline(always)]
pub(crate) fn current() -> *const Utcb {
    let utcb: *const Utcb;
    // SAFETY: only reads thread-local data through `gs`.
    unsafe {
        core::arch::asm!(
            "mov {}, gs:0",
            out(reg) utcb,
            options(nostack, preserves_flags, readonly),
        );
    }
    utcb
}
