//! L4 IPC.
//!
//! This crate re-exports the mechanisms of `l4-ipc-core` and selects
//! protocols through cargo features.  `--features base` enables the
//! protocols every L4 system provides; single protocols
//! can be enabled by name, e.g. `--features proto-log`.
//!
//! The handles of the kernel objects are in [`kobject`], the well-known
//! capabilities of the initial environment in [`base`].
#![cfg_attr(any(not(test), target_os = "none"), no_std)]

pub use l4_ipc_core::*;

macro_rules! for_each_protocol {
    ( $macro:ident !) => {
        for_each_protocol!($macro ! ( ? ));
    };

    ($macro:ident ! ( $($prefix:expr, )* ? $(, $suffix:expr)* )) => {
        for_each_protocol!(Log, Factory, Task, Kobject, Thread, RcvEndpoint, IpcGate, ThreadGroup, Scheduler => $macro ! ( $($prefix ,)* ? $(, $suffix)*));
    };

    ($current:ident $(, $next:ident)* => $macro:ident ! ( $($prefix:expr ,)* ? $(, $suffix:expr)* )) => {
       pastey::paste! {
           #[cfg(feature = "proto-" $current:snake)]
           $macro!($($prefix,)* [<$current:snake>], [<$current:camel>] $(, $suffix)*);
        }
       for_each_protocol!($($next),* => $macro ! ( $($prefix ,)* ? $(, $suffix)* ));
    };

    ( => $macro:ident ! ( $($prefix:expr ,)* ? $(, $suffix:expr)* )) => {};
}

/// prelude module.
pub mod prelude {
    #[allow(unused)]
    macro_rules! prelude {
        ($snake:ident, $camel:ident ) => {
            pub use crate::extension::$camel as _;
        };
    }
    for_each_protocol!(prelude!);
}

pub mod extension {
    #[allow(unused)]
    macro_rules! extension {
        ($snake:ident, $camel:ident) => {
            mod $snake;
            pub use $snake::$camel;
        };
    }
    for_each_protocol!(extension!);
}

pub mod protocol {
    #[allow(unused)]
    macro_rules! protocol {
        ($snake:ident, $camel:ident) => {
            pub mod $snake;
            pub use $snake::$camel;
        };
    }
    for_each_protocol!(protocol!);

    use l4_ipc_core::protocol::Protocol;

    /// A protocol whose objects an IPC gate or receive endpoint can be
    /// bound to as a receiver: a thread or a thread group.
    pub trait Bindable: Protocol {}

    #[cfg(feature = "proto-thread")]
    impl Bindable for thread::Thread {}

    #[cfg(feature = "proto-thread_group")]
    impl Bindable for thread_group::ThreadGroup {}
}

/// Handles for the kernel objects.  A handle is a capability that is typed
/// with the protocols its kernel object speaks.
pub mod kobject {
    #[allow(unused)]
    macro_rules! kobject {
        ($mod:ident, $type:ident) => {
            mod $mod;
            pub use $mod::$type;
        };
    }

    #[cfg(feature = "proto-factory")]
    kobject!(factory, Factory);

    #[cfg(feature = "proto-ipc_gate")]
    kobject!(ipc_gate, IpcGate);

    #[cfg(feature = "proto-task")]
    kobject!(task, Task);

    #[cfg(feature = "proto-thread")]
    kobject!(thread, Thread);

    #[cfg(feature = "proto-thread_group")]
    kobject!(thread_group, ThreadGroup);

    #[cfg(feature = "proto-scheduler")]
    kobject!(scheduler, Scheduler);

    #[cfg(feature = "proto-log")]
    kobject!(vcon, Vcon);
}

#[cfg(feature = "base")]
pub mod base;
