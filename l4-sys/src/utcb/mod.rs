//! The user-level thread control block (UTCB).
//!
//! Layout (see the L4Re kernel ABI), in machine words:
//!
//! | words   | content                                      |
//! |---------|----------------------------------------------|
//! | 0..=62  | message registers (MRs)                      |
//! | 63      | architecture specific                        |
//! | 64      | buffer descriptor register (BDR)             |
//! | 65..=122| buffer registers (BRs)                       |
//! | 123     | IPC error code                               |
//! | 124     | free marker                                  |
//! | 125..=127 | thread-local storage for the user          |

use crate::umword;
use core::cell::Cell;

/// Number of message registers.
pub const MR_COUNT: usize = 63;
/// Number of buffer registers.
pub const BR_COUNT: usize = 58;
/// Number of user TLS words.
pub const TLS_COUNT: usize = 3;

/// The UTCB of one thread.
///
/// The kernel reads and writes the UTCB during system calls, hence all
/// words are [`Cell`]s.  This also makes `Utcb` `!Sync`, so a reference to
/// it cannot be sent to another thread.
#[repr(C)]
pub struct Utcb {
    mr: [Cell<umword>; MR_COUNT],
    arch: Cell<umword>,
    bdr: Cell<umword>,
    br: [Cell<umword>; BR_COUNT],
    error: Cell<umword>,
    free_marker: Cell<umword>,
    tls: [Cell<umword>; TLS_COUNT],
}

const _: () = assert!(core::mem::size_of::<Utcb>() == 128 * core::mem::size_of::<umword>());

#[allow(clippy::declare_interior_mutable_const)]
const ZERO: Cell<umword> = Cell::new(0);

impl Utcb {
    /// A zero-initialised UTCB that is not connected to any thread, e.g.
    /// for testing message marshalling.
    pub const fn zeroed() -> Self {
        Utcb {
            mr: [ZERO; MR_COUNT],
            arch: ZERO,
            bdr: ZERO,
            br: [ZERO; BR_COUNT],
            error: ZERO,
            free_marker: ZERO,
            tls: [ZERO; TLS_COUNT],
        }
    }

    /// The UTCB of the calling thread.
    ///
    /// Every L4 thread has a UTCB that stays mapped for its lifetime and is
    /// only reachable by that thread (`Utcb` is `!Sync`), so the returned
    /// reference is valid and cannot leave the thread.
    #[inline(always)]
    pub fn current() -> &'static Utcb {
        // SAFETY: see above; the architecture code merely reads the
        // thread-local location that holds the UTCB address.
        unsafe { &*crate::arch::utcb::current() }
    }

    /// Message register `i`.  Panics if `i >= MR_COUNT`.
    #[inline(always)]
    pub fn mr(&self, i: usize) -> umword {
        self.mr[i].get()
    }

    /// Set message register `i`.  Panics if `i >= MR_COUNT`.
    #[inline(always)]
    pub fn set_mr(&self, i: usize, value: umword) {
        self.mr[i].set(value)
    }

    /// Buffer register `i`.  Panics if `i >= BR_COUNT`.
    #[inline(always)]
    pub fn br(&self, i: usize) -> umword {
        self.br[i].get()
    }

    /// Set buffer register `i`.  Panics if `i >= BR_COUNT`.
    #[inline(always)]
    pub fn set_br(&self, i: usize, value: umword) {
        self.br[i].set(value)
    }

    /// The buffer descriptor register.
    #[inline(always)]
    pub fn bdr(&self) -> umword {
        self.bdr.get()
    }

    /// Set the buffer descriptor register.
    #[inline(always)]
    pub fn set_bdr(&self, value: umword) {
        self.bdr.set(value)
    }

    /// The IPC error code written by the kernel when a message tag
    /// signals an error.
    #[inline(always)]
    pub fn error(&self) -> umword {
        self.error.get()
    }

    /// User thread-local word `i`.  Panics if `i >= TLS_COUNT`.
    #[inline(always)]
    pub fn tls(&self, i: usize) -> umword {
        self.tls[i].get()
    }

    /// Set user thread-local word `i`.  Panics if `i >= TLS_COUNT`.
    #[inline(always)]
    pub fn set_tls(&self, i: usize, value: umword) {
        self.tls[i].set(value)
    }
}

#[cfg(all(test, not(target_os = "none")))]
mod tests {
    use super::*;

    #[test]
    fn registers_round_trip() {
        let u = Utcb::zeroed();
        u.set_mr(0, 1);
        u.set_mr(MR_COUNT - 1, 2);
        u.set_br(BR_COUNT - 1, 3);
        u.set_tls(TLS_COUNT - 1, 4);
        assert_eq!((u.mr(0), u.mr(MR_COUNT - 1)), (1, 2));
        assert_eq!((u.br(BR_COUNT - 1), u.tls(TLS_COUNT - 1)), (3, 4));
        assert_eq!(u.error(), 0);
    }

    #[test]
    #[should_panic]
    fn mr_out_of_range_panics() {
        Utcb::zeroed().mr(MR_COUNT);
    }
}
