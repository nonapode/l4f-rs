/// Defines a set of flags that fits into four bits, e.g. the rights field of a
/// flexpage.  The meaning of the bits depends on the kind of flexpage, which is
/// why every kind has a type of its own: the write bit of a memory flexpage is
/// not the write bit of an object flexpage.
macro_rules! flags {
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
            /// No flags.
            pub const NONE: $name = $name(0);

            $($(#[$cmeta])* pub const $konst: $name = $name($value);)+

            /// The raw value of the four bits.
            pub const fn bits(self) -> u8 {
                self.0
            }

            /// The flags for the raw value of the field; `None` if it does
            /// not fit into the four bits.
            pub const fn from_bits(bits: u8) -> Option<$name> {
                if bits <= 0xf {
                    Some($name(bits))
                } else {
                    None
                }
            }

            /// The flags of both sets.
            pub const fn union(self, other: $name) -> $name {
                $name(self.0 | other.0)
            }

            /// Whether all flags of `other` are included.
            pub const fn contains(self, other: $name) -> bool {
                self.0 & other.0 == other.0
            }
        }

        impl ::core::ops::BitOr for $name {
            type Output = $name;

            fn bitor(self, rhs: $name) -> $name {
                self.union(rhs)
            }
        }

        impl ::core::ops::BitOrAssign for $name {
            fn bitor_assign(&mut self, rhs: $name) {
                *self = self.union(rhs);
            }
        }

        impl ::core::fmt::Debug for $name {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
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

        impl ::l4_bitfield::Encode<::l4_sys::umword, 4> for $name {
            fn encode(&self) -> ::l4_bitfield::Bits<::l4_sys::umword, 4> {
                ::l4_bitfield::Bits::new(self.0 as ::l4_sys::umword)
            }
        }

        impl ::l4_bitfield::Decode<::l4_sys::umword, 4> for $name {
            /// Keeps bits that have no name, so that a flexpage survives
            /// being decoded and encoded again.
            fn decode(bits: ::l4_bitfield::Bits<::l4_sys::umword, 4>) -> Self {
                $name(bits.into_inner() as u8)
            }
        }
    };
}

pub(super) use flags;
