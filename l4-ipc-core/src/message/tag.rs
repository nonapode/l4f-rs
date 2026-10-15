use l4_sys::syscall::{Input, Marker, Output};
use l4_sys::{mword, umword};

/// Flags of a message tag (bits 12..=15).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct TagFlags(umword);

impl TagFlags {
    /// No flags.
    pub const NONE: TagFlags = TagFlags(0);
    /// Transfer the FPU state with the message (`L4_MSGTAG_TRANSFER_FPU`).
    pub const TRANSFER_FPU: TagFlags = TagFlags(0x1000);
    /// Schedule on IPC (`L4_MSGTAG_SCHEDULE`).
    pub const SCHEDULE: TagFlags = TagFlags(0x2000);
    /// Set by the kernel when an IPC failed (`L4_MSGTAG_ERROR`).
    pub const ERROR: TagFlags = TagFlags(0x8000);

    const MASK: umword = 0xf000;

    /// The union of both flag sets.
    pub const fn or(self, other: TagFlags) -> TagFlags {
        TagFlags(self.0 | other.0)
    }

    /// `true` if all flags of `other` are set in `self`.
    pub const fn contains(self, other: TagFlags) -> bool {
        self.0 & other.0 == other.0
    }

    /// The raw flag bits.
    pub const fn bits(self) -> umword {
        self.0
    }
}

/// A message tag (`l4_msgtag_t`).
///
/// Layout: bits 0..=5 number of untyped words, 6..=11 number of typed items,
/// 12..=15 flags, 16 and above the label, which carries the protocol on
/// requests and return values on replies.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Tag(mword);

impl Tag {
    /// Build a tag.  `words` and `items` are truncated to six bits and only
    /// the flag bits of `flags` are used.
    pub const fn new(label: mword, words: usize, items: usize, flags: TagFlags) -> Tag {
        let label = ((label as umword) << 16) as mword;
        Tag(label
            | (words & 0x3f) as mword
            | ((items & 0x3f) << 6) as mword
            | (flags.0 & TagFlags::MASK) as mword)
    }

    /// Wrap a raw tag value.
    pub const fn from_raw(raw: mword) -> Tag {
        Tag(raw)
    }

    /// The raw tag value.
    pub const fn raw(self) -> mword {
        self.0
    }

    /// The label (sign-extended).
    pub const fn label(self) -> mword {
        self.0 >> 16
    }

    /// Number of untyped message words.
    pub const fn words(self) -> usize {
        (self.0 & 0x3f) as usize
    }

    /// Number of typed message items.
    pub const fn items(self) -> usize {
        ((self.0 >> 6) & 0x3f) as usize
    }

    /// The flags.
    pub const fn flags(self) -> TagFlags {
        TagFlags(self.0 as umword & TagFlags::MASK)
    }

    /// `true` if the kernel flagged an IPC error.
    pub const fn has_error(self) -> bool {
        self.flags().contains(TagFlags::ERROR)
    }
}

impl Input for Tag {
    fn encode(&self, _: Marker) -> umword {
        self.0 as umword
    }
}

impl Output for Tag {
    fn decode(value: umword, _: Marker) -> Self {
        Tag(value as mword)
    }
}

#[cfg(all(test, not(target_os = "none")))]
mod tests {
    use super::*;

    #[test]
    fn fields_round_trip() {
        let t = Tag::new(-13, 3, 2, TagFlags::SCHEDULE);
        assert_eq!(t.label(), -13);
        assert_eq!(t.words(), 3);
        assert_eq!(t.items(), 2);
        assert_eq!(t.flags(), TagFlags::SCHEDULE);
        assert!(!t.has_error());
    }

    #[test]
    fn bit_layout() {
        let t = Tag::new(1, 5, 1, TagFlags::TRANSFER_FPU.or(TagFlags::SCHEDULE));
        assert_eq!(t.raw() & 0xffff, 0x3000 | (1 << 6) | 5);
        assert_eq!(t.raw() >> 16, 1);
    }

    #[test]
    fn error_flag() {
        assert!(Tag::from_raw(0x8000).has_error());
        assert!(!Tag::default().has_error());
    }

    #[test]
    fn fields_are_masked() {
        let t = Tag::new(0, 64, 64, TagFlags::NONE);
        assert_eq!((t.words(), t.items()), (0, 0));
    }
}
