use core::time::Duration;
use l4_sys::syscall::{Input, Marker};
use l4_sys::umword;

/// An IPC timeout pair (`l4_timeout_t`): the send timeout in the upper and
/// the receive timeout in the lower 16 bits.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Timeout(u32);

impl Timeout {
    /// Wait forever in both phases (`L4_IPC_NEVER`).
    pub const NEVER: Timeout = Timeout::new(Spec::NEVER, Spec::NEVER);

    /// Do not wait in either phase.
    pub const ZERO: Timeout = Timeout::new(Spec::ZERO, Spec::ZERO);

    /// Combine the 16-bit send and receive timeouts.
    pub const fn new(send: Spec, receive: Spec) -> Timeout {
        let send = send.0 as u32;
        let receive = receive.0 as u32;
        Self((send << 16) | receive)
    }

    /// Wrap a raw timeout value.
    pub const fn from_raw(raw: u32) -> Timeout {
        Self(raw)
    }

    /// The raw timeout value.
    pub const fn raw(self) -> u32 {
        self.0
    }
}

impl Input for Timeout {
    fn encode(&self, _: Marker) -> umword {
        self.0 as umword
    }
}

/// An IPC timeout specification
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Spec(u16);

pub enum SpecError {
    Overflow,
    Underflow,
}

impl Spec {
    const MAN_MASK: u16 = 0x03FF;
    const EXP_MASK: u16 = 0x7C00;
    const EXP_SHIFT: usize = 10;

    /// One 16-bit timeout meaning "never".
    pub const NEVER: Self = Self::from_raw(0);

    /// One 16-bit timeout meaning "do not wait" (only bit 10 set).
    pub const ZERO: Self = Self::from_raw(1 << 10);

    pub const MAX_TIMEOUT: Duration = Duration::from_micros(1023 * (1u64 << 31));

    pub fn from_duration(duration: Duration) -> Result<Self, SpecError> {
        if duration.is_zero() {
            return Ok(Self::ZERO);
        } else if duration > Self::MAX_TIMEOUT {
            return Err(SpecError::Overflow);
        }

        let micros = duration.as_micros();

        // Sub-microsecond durations cannot be represented positively
        if micros == 0 {
            return Err(SpecError::Underflow);
        }

        let mut exp = 0u8;
        while (micros >> exp) > Self::MAN_MASK.into() {
            exp += 1;
        }

        let man = (micros >> exp) as u16;
        let exp = exp as u16;
        Ok(Self(
            (man & Self::MAN_MASK) | ((exp << Self::EXP_SHIFT) & Self::EXP_MASK),
        ))
    }

    pub const fn from_raw(value: u16) -> Self {
        Self(value)
    }
}

#[cfg(all(test, not(target_os = "none")))]
mod tests {
    use super::*;

    #[test]
    fn special_values() {
        assert_eq!(Timeout::NEVER.raw(), 0);
        assert_eq!(Timeout::ZERO.raw(), 0x0400_0400);
        assert_eq!(
            Timeout::new(Spec::from_raw(1), Spec::from_raw(2)).raw(),
            0x0001_0002
        );
    }
}
