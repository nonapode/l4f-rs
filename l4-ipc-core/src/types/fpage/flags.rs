/// Defines a set of flags that fits into four bits, e.g. the rights field of a
/// flexpage.  The meaning of the bits depends on the kind of flexpage, which
/// is why every kind has a type of its own: the write bit of a memory flexpage
/// is not the write bit of an object flexpage.
macro_rules! flags {
    (
        $(#[$meta:meta])*
        $name:ident {
            $($(#[$cmeta:meta])* $konst:ident = $value:expr,)+
        }
    ) => {
        $(#[$meta])*
        #[derive(Clone, Copy, PartialEq, Eq, Hash)]
        pub struct $name(arbitrary_int::u4);

        impl $name {
            /// No flags.
            pub const NONE: $name = $name(arbitrary_int::u4::new(0));

            $($(#[$cmeta])* pub const $konst: $name = $name(arbitrary_int::u4::new($value));)+

            /// The raw value of the four bits.
            pub const fn bits(self) -> arbitrary_int::u4 {
                self.0
            }

            /// The flags of the raw field; every four bit value is valid and
            /// kept, so that a flexpage survives being decoded and encoded
            /// again.
            pub const fn from_bits(bits: arbitrary_int::u4) -> $name {
                $name(bits)
            }

            /// The flags of both sets.
            pub const fn union(self, other: $name) -> $name {
                $name(arbitrary_int::u4::new(self.0.value() | other.0.value()))
            }

            /// Whether all flags of `other` are included.
            pub const fn contains(self, other: $name) -> bool {
                self.0.value() & other.0.value() == other.0.value()
            }
        }

        impl ::core::default::Default for $name {
            fn default() -> Self {
                Self::NONE
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
                let bits = self.0.value();

                f.write_str(concat!(stringify!($name), "("))?;
                $(
                    if bits & $value != 0 {
                        f.write_str(separator)?;
                        f.write_str(stringify!($konst))?;
                        separator = "|";
                    }
                )+
                let unknown = bits & !(0 $(| $value)+);
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
    };
}

pub(super) use flags;
