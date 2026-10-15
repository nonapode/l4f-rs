use l4_sys::umword;

/// Syscall flags, OR-ed into the destination capability selector
/// (`l4_syscall_flags_t`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct Flags(umword);

impl Flags {
    /// Send phase.
    pub(crate) const SEND: Flags = Flags(0x1);
    /// Receive phase.
    pub(crate) const RECV: Flags = Flags(0x2);
    /// The receive phase is an open wait.
    pub(crate) const OPEN_WAIT: Flags = Flags(0x4);
    /// Use the reply capability of the current thread.
    pub(crate) const REPLY: Flags = Flags(0x8);

    pub(crate) const CALL: Flags = Flags::SEND.or(Flags::RECV);
    pub(crate) const WAIT: Flags = Flags::OPEN_WAIT.or(Flags::RECV);
    pub(crate) const SEND_AND_WAIT: Flags = Flags::OPEN_WAIT.or(Flags::CALL);
    pub(crate) const REPLY_AND_WAIT: Flags = Flags::WAIT.or(Flags::SEND).or(Flags::REPLY);

    pub(crate) const fn or(self, other: Flags) -> Flags {
        Flags(self.0 | other.0)
    }

    pub(crate) const fn bits(self) -> umword {
        self.0
    }
}

#[cfg(all(test, not(target_os = "none")))]
mod tests {
    use super::*;

    #[test]
    fn combined_flags() {
        assert_eq!(Flags::CALL.bits(), 0b0011);
        assert_eq!(Flags::WAIT.bits(), 0b0110);
        assert_eq!(Flags::SEND_AND_WAIT.bits(), 0b0111);
        assert_eq!(Flags::REPLY_AND_WAIT.bits(), 0b1111);
    }
}
