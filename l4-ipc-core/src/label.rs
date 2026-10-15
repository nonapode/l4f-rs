use l4_sys::syscall::{Input, Marker, Output};
use l4_sys::umword;

/// An IPC label: the badge of the IPC gate a message came through, or the
/// label to send when addressing a thread.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub struct Label(umword);

impl Label {
    /// Wrap a raw label.
    pub const fn new(raw: umword) -> Label {
        Label(raw)
    }

    /// The raw label.
    pub const fn get(self) -> umword {
        self.0
    }
}

impl Input for Label {
    fn encode(&self, _: Marker) -> umword {
        self.0
    }
}

impl Output for Label {
    fn decode(value: umword, _: Marker) -> Self {
        Label(value)
    }
}
