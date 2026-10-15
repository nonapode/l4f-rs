//! Typed bitfields: a bitfield is a storage word, a field is a bit range with
//! a codec that converts between the bits and a value type.
//!
//! The bounds of the field traits are generic constant expressions, so a
//! crate that reads or writes fields needs `#![feature(generic_const_exprs)]`
//! as well.
#![cfg_attr(any(not(test), target_os = "none"), no_std)]
#![feature(generic_const_exprs)]
#![allow(incomplete_features)]

use core::marker::PhantomData;

// ============================================================
// Marker and storage abstraction
// ============================================================

pub struct Marker(PhantomData<fn() -> ()>);

impl Marker {
    fn new() -> Self {
        Self(PhantomData)
    }
}

pub trait Word: Copy + Eq {
    const BITS: usize;

    fn zero() -> Self;
    fn ones() -> Self;
    fn and(self, rhs: Self) -> Self;
    fn or(self, rhs: Self) -> Self;
    fn not(self) -> Self;
    fn shl(self, shift: usize) -> Self;
    fn shr(self, shift: usize) -> Self;

    fn mask(width: usize) -> Self {
        if width == 0 {
            Self::zero()
        } else if width >= Self::BITS {
            Self::ones()
        } else {
            Self::ones().shr(Self::BITS - width)
        }
    }
}

macro_rules! impl_word {
    ($($t:ty),* $(,)?) => {
        $(
            impl Word for $t {
                const BITS: usize = <$t>::BITS as usize;

                fn zero() -> Self { 0 }
                fn ones() -> Self { !0 }
                fn and(self, rhs: Self) -> Self { self & rhs }
                fn or(self, rhs: Self) -> Self { self | rhs }
                fn not(self) -> Self { !self }

                fn shl(self, shift: usize) -> Self {
                    if shift >= <Self as Word>::BITS {
                        0
                    } else {
                        self << shift
                    }
                }

                fn shr(self, shift: usize) -> Self {
                    if shift >= <Self as Word>::BITS {
                        0
                    } else {
                        self >> shift
                    }
                }
            }
        )*
    };
}

impl_word!(u8, u16, u32, u64, u128, usize);

pub trait Bitfield: Sized {
    type Type: Word;

    fn from_raw(value: Self::Type, _: Marker) -> Self;
    fn raw(&self, _: Marker) -> &Self::Type;
}

// ============================================================
// Unshifted and positioned bit representations
// ============================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bits<T, const N: usize>(T);

impl<T, const N: usize> Bits<T, N> {
    pub fn new(value: T) -> Self {
        Self(value)
    }

    pub fn into_inner(self) -> T {
        self.0
    }

    pub fn as_inner(&self) -> &T {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bitmask<T, const LHS: usize, const RHS: usize>(T);

impl<T, const LHS: usize, const RHS: usize> Bitmask<T, LHS, RHS> {
    pub fn new(value: T) -> Self {
        Self(value)
    }

    pub fn into_inner(self) -> T {
        self.0
    }

    pub fn as_inner(&self) -> &T {
        &self.0
    }
}

impl<T: Word, const LHS: usize, const RHS: usize> Bitmask<T, LHS, RHS>
where
    [(); RHS - LHS]:,
{
    pub fn into_bits(self) -> Bits<T, { RHS - LHS }> {
        let value = self.0.shr(LHS).and(T::mask(RHS - LHS));
        Bits::new(value)
    }
}

// ============================================================
// Generic value conversions
// ============================================================
pub enum Error<T> {
    InvalidValue(T),
}

pub trait Encode<T, const BITS: usize> {
    fn encode(&self) -> Bits<T, BITS>;
}

pub trait TryEncode<T, const BITS: usize> {
    type Error;

    fn try_encode(&self) -> Result<Bits<T, BITS>, Self::Error>;
}

pub trait Decode<T, const BITS: usize>: Sized {
    fn decode(bits: Bits<T, BITS>) -> Self;
}

pub trait TryDecode<T, const BITS: usize>: Sized {
    type Error;

    fn try_decode(bits: Bits<T, BITS>) -> Result<Self, Self::Error>;
}

// ============================================================
// Position-aware conversion traits
//
// The implementor is a codec strategy. Value is the type being
// encoded or decoded. LHS..RHS is a half-open bit range.
// ============================================================

pub trait Encoder<B: Bitfield, const LHS: usize, const RHS: usize> {
    type Value;

    fn encode(value: &Self::Value, marker: Marker) -> Bitmask<B::Type, LHS, RHS>;
}

pub trait TryEncoder<B: Bitfield, const LHS: usize, const RHS: usize> {
    type Value;
    type Error;

    fn try_encode(
        value: &Self::Value,
        marker: Marker,
    ) -> Result<Bitmask<B::Type, LHS, RHS>, Self::Error>;
}

pub trait Decoder<B: Bitfield, const LHS: usize, const RHS: usize> {
    type Value;

    fn decode(bits: Bitmask<B::Type, LHS, RHS>, marker: Marker) -> Self::Value;
}

pub trait TryDecoder<B: Bitfield, const LHS: usize, const RHS: usize> {
    type Value;
    type Error;

    fn try_decode(
        bits: Bitmask<B::Type, LHS, RHS>,
        marker: Marker,
    ) -> Result<Self::Value, Self::Error>;
}

// ============================================================
// Default codec strategies
//
// These are parameterized by V, so V is constrained by Self and
// does not need to appear as an additional trait parameter.
// ============================================================

pub struct DefaultEncoding<V>(PhantomData<fn() -> V>);
pub struct DefaultDecoding<V>(PhantomData<fn() -> V>);

impl<B, V, const LHS: usize, const RHS: usize> Encoder<B, LHS, RHS> for DefaultEncoding<V>
where
    B: Bitfield,
    V: Encode<B::Type, { RHS - LHS }>,
    [(); RHS - LHS]:,
{
    type Value = V;

    fn encode(value: &V, _: Marker) -> Bitmask<B::Type, LHS, RHS> {
        let raw = value.encode().into_inner();
        let raw = raw.and(B::Type::mask(RHS - LHS));
        Bitmask::new(raw.shl(LHS))
    }
}

impl<B, V, const LHS: usize, const RHS: usize> TryEncoder<B, LHS, RHS> for DefaultEncoding<V>
where
    B: Bitfield,
    V: TryEncode<B::Type, { RHS - LHS }>,
    [(); RHS - LHS]:,
{
    type Value = V;
    type Error = V::Error;

    fn try_encode(value: &V, _: Marker) -> Result<Bitmask<B::Type, LHS, RHS>, Self::Error> {
        let raw = value.try_encode()?.into_inner();
        let raw = raw.and(B::Type::mask(RHS - LHS));
        Ok(Bitmask::new(raw.shl(LHS)))
    }
}

impl<B, V, const LHS: usize, const RHS: usize> Decoder<B, LHS, RHS> for DefaultDecoding<V>
where
    B: Bitfield,
    V: Decode<B::Type, { RHS - LHS }>,
    [(); RHS - LHS]:,
{
    type Value = V;

    fn decode(bits: Bitmask<B::Type, LHS, RHS>, _: Marker) -> V {
        V::decode(bits.into_bits())
    }
}

impl<B, V, const LHS: usize, const RHS: usize> TryDecoder<B, LHS, RHS> for DefaultDecoding<V>
where
    B: Bitfield,
    V: TryDecode<B::Type, { RHS - LHS }>,
    [(); RHS - LHS]:,
{
    type Value = V;
    type Error = V::Error;

    fn try_decode(bits: Bitmask<B::Type, LHS, RHS>, _: Marker) -> Result<V, Self::Error> {
        V::try_decode(bits.into_bits())
    }
}

// ============================================================
// Codec selection
//
// One Codec type chooses all four strategies. A custom codec
// can use DefaultEncoding<V> or DefaultDecoding<V> for either
// direction independently.
// ============================================================

pub struct DefaultCodec;

pub trait Codec<B: Bitfield, V, const LHS: usize, const RHS: usize> {
    type Encoder;
    type Decoder;
    type TryEncoder;
    type TryDecoder;
}

impl<B, V, const LHS: usize, const RHS: usize> Codec<B, V, LHS, RHS> for DefaultCodec
where
    B: Bitfield,
{
    type Encoder = DefaultEncoding<V>;
    type Decoder = DefaultDecoding<V>;
    type TryEncoder = DefaultEncoding<V>;
    type TryDecoder = DefaultDecoding<V>;
}

// ============================================================
// Field descriptor
//
// A field is a type, never a value: name it with an alias and
// access it through `Access`, e.g.
//
//     type Enabled = Field<Flags, bool, 0, 1>;
//     flags.set::<Enabled>(&true);
//     let enabled = flags.get::<Enabled>();
// ============================================================

/// The bit range `LHS..RHS` of the bitfield `B`, holding a value of type
/// `V` that is converted by the codec `C`.
///
/// The type cannot be instantiated.  It only names a field, which is read
/// and written through [`Access`].
pub enum Field<B, V, const LHS: usize, const RHS: usize, C = DefaultCodec> {
    #[doc(hidden)]
    Never(core::convert::Infallible, PhantomData<fn() -> (B, V, C)>),
}

impl<B, V, const LHS: usize, const RHS: usize, C> Field<B, V, LHS, RHS, C>
where
    B: Bitfield,
{
    /// Evaluated when a field is used: the range must be valid and fit into
    /// the storage word.
    const RANGE_IS_VALID: () = assert!(
        LHS <= RHS && RHS <= <B::Type as Word>::BITS,
        "invalid field range"
    );

    fn mask() -> B::Type {
        let () = Self::RANGE_IS_VALID;
        B::Type::mask(RHS - LHS).shl(LHS)
    }

    fn masked(bitfield: &B) -> Bitmask<B::Type, LHS, RHS> {
        Bitmask::new(bitfield.raw(Marker::new()).and(Self::mask()))
    }

    fn replace(bitfield: &mut B, encoded: B::Type) {
        let raw = *bitfield.raw(Marker::new());
        let cleared = raw.and(Self::mask().not());
        let updated = cleared.or(encoded.and(Self::mask()));

        *bitfield = B::from_raw(updated, Marker::new());
    }
}

/// A field that can be read.
pub trait Read {
    type Bitfield: Bitfield;
    type Value;

    fn read(bitfield: &Self::Bitfield) -> Self::Value;
}

/// A field that can be read, but not every bit pattern is a valid value.
pub trait TryRead {
    type Bitfield: Bitfield;
    type Value;
    type Error;

    fn try_read(bitfield: &Self::Bitfield) -> Result<Self::Value, Self::Error>;
}

/// A field that can be written.
pub trait Write {
    type Bitfield: Bitfield;
    type Value;

    fn write(bitfield: &mut Self::Bitfield, value: &Self::Value);
}

/// A field that can be written, but not every value can be encoded.  The
/// bitfield is left unchanged on an error.
pub trait TryWrite {
    type Bitfield: Bitfield;
    type Value;
    type Error;

    fn try_write(bitfield: &mut Self::Bitfield, value: &Self::Value) -> Result<(), Self::Error>;
}

impl<B, V, const LHS: usize, const RHS: usize, C> Read for Field<B, V, LHS, RHS, C>
where
    B: Bitfield,
    [(); RHS - LHS]:,
    C: Codec<B, V, LHS, RHS>,
    C::Decoder: Decoder<B, LHS, RHS, Value = V>,
{
    type Bitfield = B;
    type Value = V;

    fn read(bitfield: &B) -> V {
        <C::Decoder as Decoder<B, LHS, RHS>>::decode(Self::masked(bitfield), Marker::new())
    }
}

impl<B, V, const LHS: usize, const RHS: usize, C> TryRead for Field<B, V, LHS, RHS, C>
where
    B: Bitfield,
    [(); RHS - LHS]:,
    C: Codec<B, V, LHS, RHS>,
    C::TryDecoder: TryDecoder<B, LHS, RHS, Value = V>,
{
    type Bitfield = B;
    type Value = V;
    type Error = <C::TryDecoder as TryDecoder<B, LHS, RHS>>::Error;

    fn try_read(bitfield: &B) -> Result<V, Self::Error> {
        <C::TryDecoder as TryDecoder<B, LHS, RHS>>::try_decode(
            Self::masked(bitfield),
            Marker::new(),
        )
    }
}

impl<B, V, const LHS: usize, const RHS: usize, C> Write for Field<B, V, LHS, RHS, C>
where
    B: Bitfield,
    [(); RHS - LHS]:,
    C: Codec<B, V, LHS, RHS>,
    C::Encoder: Encoder<B, LHS, RHS, Value = V>,
{
    type Bitfield = B;
    type Value = V;

    fn write(bitfield: &mut B, value: &V) {
        let encoded =
            <C::Encoder as Encoder<B, LHS, RHS>>::encode(value, Marker::new()).into_inner();

        Self::replace(bitfield, encoded);
    }
}

impl<B, V, const LHS: usize, const RHS: usize, C> TryWrite for Field<B, V, LHS, RHS, C>
where
    B: Bitfield,
    [(); RHS - LHS]:,
    C: Codec<B, V, LHS, RHS>,
    C::TryEncoder: TryEncoder<B, LHS, RHS, Value = V>,
{
    type Bitfield = B;
    type Value = V;
    type Error = <C::TryEncoder as TryEncoder<B, LHS, RHS>>::Error;

    fn try_write(bitfield: &mut B, value: &V) -> Result<(), Self::Error> {
        let encoded = <C::TryEncoder as TryEncoder<B, LHS, RHS>>::try_encode(value, Marker::new())?
            .into_inner();

        Self::replace(bitfield, encoded);
        Ok(())
    }
}

/// Reads and writes the fields of a bitfield; import it to call
/// `bitfield.get::<Field>()` and friends.
pub trait Access: Bitfield + Sized {
    fn get<F: Read<Bitfield = Self>>(&self) -> F::Value {
        F::read(self)
    }

    fn try_get<F: TryRead<Bitfield = Self>>(&self) -> Result<F::Value, F::Error> {
        F::try_read(self)
    }

    fn set<F: Write<Bitfield = Self>>(&mut self, value: &F::Value) -> &mut Self {
        F::write(self, value);
        self
    }

    fn try_set<F: TryWrite<Bitfield = Self>>(
        &mut self,
        value: &F::Value,
    ) -> Result<&mut Self, F::Error> {
        F::try_write(self, value)?;
        Ok(self)
    }
}

impl<B: Bitfield> Access for B {}

#[cfg(all(test, not(target_os = "none")))]
mod tests {
    extern crate std;
    use super::*;

    // ============================================================
    // Example: ordinary conversions
    // ============================================================

    impl Encode<u8, 1> for bool {
        fn encode(&self) -> Bits<u8, 1> {
            Bits::new(u8::from(*self))
        }
    }

    impl Decode<u8, 1> for bool {
        fn decode(bits: Bits<u8, 1>) -> Self {
            bits.into_inner() != 0
        }
    }

    impl Encode<u8, 2> for u8 {
        fn encode(&self) -> Bits<u8, 2> {
            Bits::new(*self)
        }
    }

    impl Decode<u8, 2> for u8 {
        fn decode(bits: Bits<u8, 2>) -> Self {
            bits.into_inner()
        }
    }

    // ============================================================
    // Example bitfield
    // ============================================================

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct Flags(u8);

    impl Bitfield for Flags {
        type Type = u8;

        fn from_raw(value: u8, _: Marker) -> Self {
            Self(value)
        }

        fn raw(&self, _: Marker) -> &u8 {
            &self.0
        }
    }

    pub type Enabled = Field<Flags, bool, 0, 1>;

    pub type Mode2 = Field<Flags, u8, 1, 3>;

    // ============================================================
    // Example custom decoder with default encoding
    // ============================================================

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Mode {
        Off,
        On,
    }

    impl Encode<u8, 2> for Mode {
        fn encode(&self) -> Bits<u8, 2> {
            Bits::new(match self {
                Mode::Off => 0,
                Mode::On => 1,
            })
        }
    }

    pub struct ModeCodec;

    impl Codec<Flags, Mode, 1, 3> for ModeCodec {
        type Encoder = DefaultEncoding<Mode>;
        type Decoder = ModeDecoder;
        type TryEncoder = DefaultEncoding<Mode>;
        type TryDecoder = ModeDecoder;
    }

    pub struct ModeDecoder;

    impl Decoder<Flags, 1, 3> for ModeDecoder {
        type Value = Mode;

        fn decode(bits: Bitmask<u8, 1, 3>, _: Marker) -> Mode {
            match bits.into_bits().into_inner() {
                0 => Mode::Off,
                _ => Mode::On,
            }
        }
    }

    impl TryDecoder<Flags, 1, 3> for ModeDecoder {
        type Value = Mode;
        type Error = &'static str;

        fn try_decode(bits: Bitmask<u8, 1, 3>, _: Marker) -> Result<Mode, Self::Error> {
            match bits.into_bits().into_inner() {
                0 => Ok(Mode::Off),
                1 => Ok(Mode::On),
                _ => Err("invalid mode"),
            }
        }
    }

    pub type CustomMode = Field<Flags, Mode, 1, 3, ModeCodec>;

    // ============================================================
    // Example fallible encoding
    // ============================================================

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct Level(u8);

    impl TryEncode<u8, 2> for Level {
        type Error = &'static str;

        fn try_encode(&self) -> Result<Bits<u8, 2>, Self::Error> {
            match self.0 {
                0..=2 => Ok(Bits::new(self.0)),
                _ => Err("level out of range"),
            }
        }
    }

    impl TryDecode<u8, 2> for Level {
        type Error = &'static str;

        fn try_decode(bits: Bits<u8, 2>) -> Result<Self, Self::Error> {
            match bits.into_inner() {
                0..=2 => Ok(Level(bits.into_inner())),
                _ => Err("level out of range"),
            }
        }
    }

    pub type LevelBits = Field<Flags, Level, 3, 5>;

    // ============================================================
    // Demonstration
    // ============================================================

    #[test]
    fn reads_and_writes_fields() {
        let mut flags = Flags(0);

        flags.set::<Enabled>(&true).set::<Mode2>(&2);

        assert!(flags.get::<Enabled>());
        assert_eq!(flags.get::<Mode2>(), 2);
        assert_eq!(flags.0, 0b101);
    }

    #[test]
    fn custom_decoder_is_selected() {
        let mut flags = Flags(0);

        flags.set::<CustomMode>(&Mode::On);

        assert_eq!(flags.get::<CustomMode>(), Mode::On);
        assert_eq!(flags.try_get::<CustomMode>(), Ok(Mode::On));
    }

    #[test]
    fn try_decode_can_reject_invalid_values() {
        let flags = Flags(0b110);

        assert_eq!(flags.try_get::<CustomMode>(), Err("invalid mode"));
    }

    #[test]
    fn setters_preserve_unrelated_bits() {
        let mut flags = Flags(0b1111_1000);

        flags.set::<Enabled>(&true);

        assert_eq!(flags.0, 0b1111_1001);
    }

    #[test]
    fn try_set_writes_valid_values() {
        let mut flags = Flags(0b0000_0001);

        assert!(flags.try_set::<LevelBits>(&Level(2)).is_ok());

        assert_eq!(flags.0, 0b0001_0001);
        assert_eq!(flags.try_get::<LevelBits>(), Ok(Level(2)));
    }

    #[test]
    fn try_set_leaves_the_bitfield_alone_on_error() {
        let mut flags = Flags(0b1010_1010);

        assert_eq!(
            flags.try_set::<LevelBits>(&Level(3)).map(|_| ()),
            Err("level out of range")
        );

        assert_eq!(flags.0, 0b1010_1010);
    }

    #[test]
    fn fields_are_independent_of_the_value_width() {
        let mut flags = Flags(0xff);

        flags.set::<Mode2>(&0);

        assert_eq!(flags.0, 0b1111_1001);
    }
}
