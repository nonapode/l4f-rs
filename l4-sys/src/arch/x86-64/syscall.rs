//! x86-64 syscall ABI (Fiasco.OC): `syscall` instruction.
//!
//! Register assignment, in the generic argument order
//! `(tag, label, dest | flags, timeout)`:
//!
//! | argument | register | in | out |
//! |----------|----------|----|-----|
//! | tag      | rax      | x  | x   |
//! | label    | rsi      | x  | x   |
//! | dest     | rdx      | x  |     |
//! | timeout  | r8       | x  |     |
//!
//! The kernel additionally clobbers rdi, rcx, r11 (hardware) and r15.

/// Hand the architecture's syscall register list to `$m`:
/// `$m!([ <registers> ] <rest>)`.
macro_rules! register {
    ($m:path, $($rest:tt)*) => {
        $m!([ "rax", "rsi", "rdx", "r8" ] $($rest)*)
    };
}
pub(crate) use register;

/// Issue the syscall with the given `asm!` operands.
macro_rules! asm {
    ($($operands:tt)*) => {
        unsafe {
            core::arch::asm!(
                "syscall",
                $($operands)*
                lateout("rdi") _,
                lateout("rcx") _,
                lateout("r11") _,
                lateout("r15") _,
            )
        }
    };
}
pub(crate) use asm;
