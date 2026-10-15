use super::operation::invoke_raw;
use super::{Operation, Protocol};
use crate::{CapIdx, Error, Timeout};
use l4_sys::Utcb;

/// A typed wrapper around a capability selector.
///
/// Which protocols the capability speaks is declared separately through
/// [`Speaks`], so one handle can speak several protocols (a console speaks
/// the log protocol and more) and protocol crates can implement foreign
/// traits for their own handle types.  Declare handles with the
/// [`handle!`](crate::handle) macro.
pub trait Handle: Copy {
    /// The wrapped capability selector.
    fn idx(&self) -> CapIdx;

    /// Perform `Op` using the given UTCB.
    fn invoke_with<Op>(
        &self,
        utcb: &Utcb,
        request: Op::Request<'_>,
        timeout: Timeout,
    ) -> Result<Op::Reply, Error>
    where
        Op: Operation,
        Self: Speaks<Op::Proto>,
    {
        invoke_raw::<Op>(self.idx(), utcb, request, timeout)
    }

    /// Perform `Op` using the calling thread's UTCB.
    fn invoke<Op>(&self, request: Op::Request<'_>, timeout: Timeout) -> Result<Op::Reply, Error>
    where
        Op: Operation,
        Self: Speaks<Op::Proto>,
    {
        self.invoke_with::<Op>(Utcb::current(), request, timeout)
    }
}

/// The capability behind this handle implements protocol `P`.
///
/// A handle may implement this for any number of protocols.  Operations of
/// `P` can only be invoked on handles that declare it; the kernel or server
/// decides at run time whether the claim is true.
///
/// ```compile_fail,E0277
/// use l4_ipc_core::protocol::{Handle, Operation, Protocol};
/// use l4_ipc_core::{handle, Timeout};
///
/// struct A; impl Protocol for A { const PROTO: isize = 1; }
/// struct B; impl Protocol for B { const PROTO: isize = 2; }
/// struct OpB; impl Operation for OpB {
///     type Proto = B;
///     const OPCODE: Option<usize> = None;
///     type Request<'a> = ();
///     type Reply = ();
/// }
/// handle!(OnlyA: A);
///
/// fn f(h: OnlyA) { let _ = h.invoke::<OpB>((), Timeout::NEVER); }
/// ```
pub trait Speaks<P: Protocol>: Handle {}

/// An untyped capability: it is assumed to speak every protocol.
///
/// Meant for selectors whose type is not known statically; prefer a handle
/// declared with [`handle!`](crate::handle).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct RawCap(CapIdx);

impl RawCap {
    /// Wrap a capability selector.
    pub const fn new(idx: CapIdx) -> Self {
        RawCap(idx)
    }
}

impl Handle for RawCap {
    fn idx(&self) -> CapIdx {
        self.0
    }
}

impl<P: Protocol> Speaks<P> for RawCap {}

impl From<CapIdx> for RawCap {
    fn from(idx: CapIdx) -> Self {
        RawCap(idx)
    }
}

/// Declare a handle type for capabilities that speak the listed protocols.
///
/// ```
/// use l4_ipc_core::handle;
/// use l4_ipc_core::protocol::{Handle, Protocol};
/// use l4_ipc_core::CapIdx;
///
/// struct Log; impl Protocol for Log { const PROTO: isize = -13; }
/// struct Icu; impl Protocol for Icu { const PROTO: isize = -1; }
///
/// handle!(
///     /// A virtual console.
///     pub Vcon: Log, Icu
/// );
///
/// let v = Vcon::new(CapIdx::from_index(5));
/// assert_eq!(v.idx(), CapIdx::from_index(5));
/// assert_eq!(Vcon::from(CapIdx::from_index(5)), v);
/// ```
#[macro_export]
macro_rules! handle {
    ( $(#[$meta:meta])* $vis:vis $name:ident : $( $proto:ty ),+ $(,)? ) => {
        $(#[$meta])*
        #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
        $vis struct $name($crate::CapIdx);

        impl $name {
            /// Wrap a capability selector.
            $vis const fn new(idx: $crate::CapIdx) -> Self {
                $name(idx)
            }
        }

        impl ::core::convert::From<$crate::CapIdx> for $name {
            fn from(idx: $crate::CapIdx) -> Self {
                $name(idx)
            }
        }

        impl $crate::protocol::Handle for $name {
            fn idx(&self) -> $crate::CapIdx {
                self.0
            }
        }

        $( impl $crate::protocol::Speaks<$proto> for $name {} )+
    };
}
