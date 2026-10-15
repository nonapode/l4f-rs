//! Locating the current thread's UTCB on AArch64.

use crate::Utcb;

/// The UTCB pointer is held in `TPIDRRO_EL0` (`l4_utcb_direct`).
///
/// NOTE: not verified by compilation (no aarch64 core available when this
/// was written).
#[inline(always)]
pub(crate) fn current() -> *const Utcb {
    let utcb: *const Utcb;
    // SAFETY: reads a thread-local system register.
    unsafe {
        core::arch::asm!(
            "mrs {}, tpidrro_el0",
            out(reg) utcb,
            options(nostack, nomem, preserves_flags),
        );
    }
    utcb
}
