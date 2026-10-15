//! Generic system call interface.
//!
//! A system call is described by implementing [`Syscall`]: it names a tuple
//! of [`Input`]s and a tuple of [`Output`]s.  Element `i` of either tuple is
//! mapped to register `i` of the architecture's syscall register list (see
//! the private `arch::syscall::register!` macros).

mod input;
mod marker;
mod tuple;

pub use input::{Input, Output};
pub use marker::Marker;
pub use tuple::{Inputs, Outputs};

/// Counts the registers in an architecture's register list.
macro_rules! count_regs {
    ([ $( $r:tt ),+ ]) => { <[()]>::len(&[ $( $crate::syscall::count_regs!(@unit $r) ),+ ]) };
    (@unit $r:tt) => { () };
}
pub(crate) use count_regs;

/// Number of registers taking part in a syscall.
pub const NUM_REGS: usize = crate::arch::syscall::register!([crate::syscall::count_regs]);

// `syscall_impl!` below hard-codes four registers.
const _: () = assert!(NUM_REGS == 4);

/// Description of a system call.
pub trait Syscall {
    /// Values passed to the kernel; element `i` goes to register `i`.
    type Inputs: Inputs;
    /// Values returned by the kernel; element `i` is read from register `i`.
    type Outputs: Outputs;

    /// Perform the system call.
    #[inline(always)]
    fn invoke(inputs: Self::Inputs) -> Self::Outputs {
        let raw = inputs.syscall(Marker(()));
        <Self::Outputs as Outputs>::decode(raw, Marker(()))
    }
}

/// Tail-recursive worker of [`syscall!`]; walks register names, register
/// indices, argument variable names and arguments in lockstep.
///
/// All argument expressions are bound to `let`s *before* the single
/// `unsafe` block so that argument code never runs inside `unsafe`.
macro_rules! syscall_impl {
    // entry: register list and wrapped arguments
    ( [ $( $r:tt ),+ ] [ $( $a:tt ),* ] ) => {
        $crate::syscall::syscall_impl!(@go
            [ $( $r ),+ ] [ 0, 1, 2, 3 ] [ a0, a1, a2, a3 ] [ $( $a ),* ]
            [] [] out)
    };

    // argument present: bind it outside `unsafe`, in/out on its register
    (@go [ $r:tt $(, $rn:tt)* ] [ $i:tt $(, $in:tt)* ] [ $v:ident $(, $vn:ident)* ]
         [ [ $e:expr ] $(, $an:tt)* ] [ $($lets:tt)* ] [ $($ops:tt)* ] $out:ident) => {
        $crate::syscall::syscall_impl!(@go [ $($rn),* ] [ $($in),* ] [ $($vn),* ] [ $($an),* ]
            [ $($lets)* let $v: $crate::umword = $e; ]
            [ $($ops)* inlateout($r) $v => $out[$i], ]
            $out)
    };

    // arguments exhausted: output-only register
    (@go [ $r:tt $(, $rn:tt)* ] [ $i:tt $(, $in:tt)* ] [ $v:ident $(, $vn:ident)* ]
         [] [ $($lets:tt)* ] [ $($ops:tt)* ] $out:ident) => {
        $crate::syscall::syscall_impl!(@go [ $($rn),* ] [ $($in),* ] [ $($vn),* ] []
            [ $($lets)* ]
            [ $($ops)* lateout($r) $out[$i], ]
            $out)
    };

    // registers exhausted: all lets first, then the one `unsafe` block
    (@go [] [] [] [] [ $($lets:tt)* ] [ $($ops:tt)* ] $out:ident) => {{
        $($lets)*
        let mut $out = [0 as $crate::umword; $crate::syscall::NUM_REGS];
        $crate::arch::syscall::asm!($($ops)*);
        $out
    }};
}
pub(crate) use syscall_impl;

/// `syscall!(a, b, ..)`: load `a`, `b`, .. (each a `umword`) into the first
/// syscall registers, enter the kernel and return all result registers as
/// `[umword; NUM_REGS]`.
macro_rules! syscall {
    ( $( $e:expr ),* $(,)? ) => {
        $crate::arch::syscall::register!(
            [ $crate::syscall::syscall_impl ]
            [ $( [ $e ] ),* ]
        )
    };
}
pub(crate) use syscall;
