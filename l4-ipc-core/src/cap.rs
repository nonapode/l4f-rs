use l4_sys::umword;

/// A capability selector (`l4_cap_idx_t`).
///
/// Layout: bits 0..=11 are reserved for the syscall flags, bit 11 marks the
/// invalid capability and the capability index lives in bits 12 and above.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct CapIdx(umword);

impl CapIdx {
    /// Position of the capability index within a selector.
    pub const SHIFT: u32 = 12;

    const INVALID_BIT: umword = 1 << 11;

    /// The invalid capability selector (`L4_INVALID_CAP`).
    pub const INVALID: CapIdx = CapIdx(umword::MAX << 11);

    /// The selector for capability slot `index`.
    pub const fn from_index(index: umword) -> Self {
        CapIdx(index << Self::SHIFT)
    }

    /// Wrap a raw selector, e.g. one obtained from the environment.
    pub const fn from_bits(bits: umword) -> Self {
        CapIdx(bits)
    }

    /// The raw selector value.
    pub const fn bits(self) -> umword {
        self.0
    }

    /// The capability slot.
    pub const fn index(self) -> umword {
        self.0 >> Self::SHIFT
    }

    /// `false` for [`CapIdx::INVALID`].
    pub const fn is_valid(self) -> bool {
        self.0 & Self::INVALID_BIT == 0
    }
}

#[cfg(all(test, not(target_os = "none")))]
mod tests {
    use super::*;

    #[test]
    fn index_round_trip() {
        let c = CapIdx::from_index(5);
        assert_eq!(c.index(), 5);
        assert_eq!(c.bits(), 5 << 12);
        assert!(c.is_valid());
    }

    #[test]
    fn invalid_cap() {
        assert!(!CapIdx::INVALID.is_valid());
        // bit 11 and everything above it is set, the flag bits are clear
        assert_eq!(CapIdx::INVALID.bits() & 0x7ff, 0);
        assert_eq!(CapIdx::INVALID.bits() >> 11, umword::MAX >> 11);
    }
}
