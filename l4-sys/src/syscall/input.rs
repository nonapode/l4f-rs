use super::Marker;
use crate::{mword, umword};

/// A value that can be passed to the kernel in one syscall register.
///
/// Other crates implement this trait for their own types, but cannot call
/// [`encode`](Input::encode) themselves because they cannot construct a
/// [`Marker`].
pub trait Input {
    /// Encode the value into a register word.
    fn encode(&self, marker: Marker) -> umword;
}

/// A value that can be read back from one syscall register.
pub trait Output: Sized {
    /// Decode the value from a register word.
    fn decode(value: umword, marker: Marker) -> Self;
}

impl Input for umword {
    #[inline(always)]
    fn encode(&self, _: Marker) -> umword {
        *self
    }
}

impl Output for umword {
    #[inline(always)]
    fn decode(value: umword, _: Marker) -> Self {
        value
    }
}

impl Input for mword {
    #[inline(always)]
    fn encode(&self, _: Marker) -> umword {
        *self as umword
    }
}

impl Output for mword {
    #[inline(always)]
    fn decode(value: umword, _: Marker) -> Self {
        value as mword
    }
}

/// The register is present but its content is ignored.
impl Output for () {
    #[inline(always)]
    fn decode(_: umword, _: Marker) -> Self {}
}
