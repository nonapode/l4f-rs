//! Tuples of [`Input`]s / [`Output`]s: element `i` lives in register `i` of
//! the architecture's syscall register list.

use super::{syscall, Input, Marker, Output, NUM_REGS};
use crate::umword;

mod sealed {
    pub trait Sealed {}
}

/// A tuple of up to [`NUM_REGS`] [`Input`]s; element `i` is passed in
/// register `i`.  Remaining registers are not loaded.
pub trait Inputs: sealed::Sealed {
    /// Run the syscall and return the raw content of all result registers.
    #[doc(hidden)]
    fn syscall(&self, marker: Marker) -> [umword; NUM_REGS];
}

/// A tuple of up to [`NUM_REGS`] [`Output`]s; element `i` is decoded from
/// register `i` after the syscall.  Use `()` to ignore a register.
pub trait Outputs: Sized + sealed::Sealed {
    /// Decode the raw result registers.
    #[doc(hidden)]
    fn decode(values: [umword; NUM_REGS], marker: Marker) -> Self;
}

macro_rules! impl_tuples {
    ( $( ( $( ( $T:ident $n:tt ) )* ) ),* $(,)? ) => { $(
        impl< $( $T ),* > sealed::Sealed for ( $( $T, )* ) {}

        impl< $( $T: Input ),* > Inputs for ( $( $T, )* ) {
            #[inline(always)]
            #[allow(unused_variables)]
            fn syscall(&self, marker: Marker) -> [umword; NUM_REGS] {
                syscall!( $( self.$n.encode(marker) ),* )
            }
        }

        impl< $( $T: Output ),* > Outputs for ( $( $T, )* ) {
            #[inline(always)]
            #[allow(unused_variables, clippy::unused_unit)]
            fn decode(values: [umword; NUM_REGS], marker: Marker) -> Self {
                ( $( <$T as Output>::decode(values[$n], marker), )* )
            }
        }
    )* };
}

impl_tuples! {
    (),
    ((A 0)),
    ((A 0)(B 1)),
    ((A 0)(B 1)(C 2)),
    ((A 0)(B 1)(C 2)(D 3)),
}
