use core::fmt;
use core::ops::{BitOr, BitOrAssign};
use l4_bitfield::{Bits, Decode, Encode};
use l4_sys::umword;

/// Defines a set of access rights that fits into the four bit rights field
/// of a flexpage.  The meaning of the bits depends on the kind of
/// flexpage, which is why every kind has a type of its own: the write bit
/// of a memory flexpage is not the write bit of an object flexpage.
macro_rules! rights {
    (
        $(#[$meta:meta])*
        $name:ident {
            $($(#[$cmeta:meta])* $konst:ident = $value:expr,)+
        }
    ) => {
        $(#[$meta])*
        #[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
        pub struct $name(u8);

        impl $name {
            /// No rights.
            pub const NONE: $name = $name(0);

            $($(#[$cmeta])* pub const $konst: $name = $name($value);)+

            /// The raw value of the rights field.
            pub const fn bits(self) -> u8 {
                self.0
            }

            /// The rights for the raw value of the field; `None` if it does
            /// not fit into the four bits.
            pub const fn from_bits(bits: u8) -> Option<$name> {
                if bits <= 0xf {
                    Some($name(bits))
                } else {
                    None
                }
            }

            /// The rights of both sets.
            pub const fn union(self, other: $name) -> $name {
                $name(self.0 | other.0)
            }

            /// Whether all rights of `other` are included.
            pub const fn contains(self, other: $name) -> bool {
                self.0 & other.0 == other.0
            }
        }

        impl BitOr for $name {
            type Output = $name;

            fn bitor(self, rhs: $name) -> $name {
                self.union(rhs)
            }
        }

        impl BitOrAssign for $name {
            fn bitor_assign(&mut self, rhs: $name) {
                *self = self.union(rhs);
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                let mut separator = "";

                f.write_str(concat!(stringify!($name), "("))?;
                $(
                    if self.0 & $value != 0 {
                        f.write_str(separator)?;
                        f.write_str(stringify!($konst))?;
                        separator = "|";
                    }
                )+
                let unknown = self.0 & !(0 $(| $value)+);
                if unknown != 0 {
                    write!(f, "{separator}{unknown:#x}")?;
                    separator = "|";
                }
                if separator.is_empty() {
                    f.write_str("NONE")?;
                }
                f.write_str(")")
            }
        }

        impl Encode<umword, 4> for $name {
            fn encode(&self) -> Bits<umword, 4> {
                Bits::new(self.0 as umword)
            }
        }

        impl Decode<umword, 4> for $name {
            /// Keeps bits that have no name, so that a flexpage survives
            /// being decoded and encoded again.
            fn decode(bits: Bits<umword, 4>) -> Self {
                $name(bits.into_inner() as u8)
            }
        }
    };
}

rights! {
    /// The rights of a memory flexpage (`L4_FPAGE_X`, `L4_FPAGE_W` and
    /// `L4_FPAGE_RO`).
    MemRights {
        /// Executable.
        X = 1,
        /// Writable.
        W = 2,
        /// Readable.
        R = 4,
    }
}

impl MemRights {
    /// Read-write.
    pub const RW: MemRights = MemRights::R.union(MemRights::W);
    /// Read-execute.
    pub const RX: MemRights = MemRights::R.union(MemRights::X);
    /// Read-write-execute.
    pub const RWX: MemRights = MemRights::RW.union(MemRights::X);
}

rights! {
    /// The rights of an object flexpage, i.e. of a capability
    /// (`L4_CAP_FPAGE_*`).
    CapRights {
        /// Write: the capability can be used to modify the object.
        W = 1,
        /// Strong: the capability can be used to send to the object.
        S = 2,
        /// Read: the capability can be used to read the object.
        R = 4,
        /// Delete: the object can be deleted through the capability.
        D = 8,
    }
}

impl CapRights {
    /// Read-write.
    pub const RW: CapRights = CapRights::R.union(CapRights::W);
    /// Read-strong.
    pub const RS: CapRights = CapRights::R.union(CapRights::S);
    /// Read-write-strong.
    pub const RWS: CapRights = CapRights::RW.union(CapRights::S);
    /// Read-write-strong-delete.
    pub const RWSD: CapRights = CapRights::RWS.union(CapRights::D);
    /// Read-write-delete.
    pub const RWD: CapRights = CapRights::RW.union(CapRights::D);
    /// Read-strong-delete.
    pub const RSD: CapRights = CapRights::RS.union(CapRights::D);
}

#[cfg(all(test, not(target_os = "none")))]
mod tests {
    use super::*;

    #[test]
    fn memory_rights_match_the_abi() {
        assert_eq!(MemRights::X.bits(), 1);
        assert_eq!(MemRights::W.bits(), 2);
        assert_eq!(MemRights::R.bits(), 4);
        assert_eq!(MemRights::RW.bits(), 6);
        assert_eq!(MemRights::RX.bits(), 5);
        assert_eq!(MemRights::RWX.bits(), 7);
    }

    #[test]
    fn capability_rights_match_the_abi() {
        assert_eq!(CapRights::W.bits(), 1);
        assert_eq!(CapRights::S.bits(), 2);
        assert_eq!(CapRights::R.bits(), 4);
        assert_eq!(CapRights::D.bits(), 8);
        assert_eq!(CapRights::RW.bits(), 5);
        assert_eq!(CapRights::RS.bits(), 6);
        assert_eq!(CapRights::RWS.bits(), 7);
        assert_eq!(CapRights::RWSD.bits(), 15);
        assert_eq!(CapRights::RWD.bits(), 13);
        assert_eq!(CapRights::RSD.bits(), 14);
    }

    #[test]
    fn the_two_kinds_do_not_mix() {
        // Both have a write right, but it is not the same bit.
        assert_ne!(MemRights::W.bits(), CapRights::W.bits());
    }

    #[test]
    fn rights_combine() {
        let mut rights = CapRights::R;
        rights |= CapRights::S;
        assert_eq!(rights, CapRights::RS);
        assert_eq!(CapRights::R | CapRights::W, CapRights::RW);
        assert!(CapRights::RWS.contains(CapRights::RS));
        assert!(!CapRights::RS.contains(CapRights::W));
        assert_eq!(CapRights::default(), CapRights::NONE);
    }

    #[test]
    fn raw_bits_are_checked() {
        assert_eq!(CapRights::from_bits(0xf), Some(CapRights::RWSD));
        assert_eq!(CapRights::from_bits(0x10), None);
    }

    #[test]
    fn codec_keeps_every_bit() {
        for bits in 0..16 {
            let rights = MemRights::from_bits(bits).unwrap();
            assert_eq!(MemRights::decode(rights.encode()), rights);
        }
    }

    #[test]
    fn debug_names_the_rights() {
        extern crate std;
        use std::format;

        assert_eq!(format!("{:?}", MemRights::RW), "MemRights(W|R)");
        assert_eq!(format!("{:?}", CapRights::NONE), "CapRights(NONE)");
        assert_eq!(
            format!("{:?}", MemRights::from_bits(8).unwrap()),
            "MemRights(0x8)"
        );
    }
}
