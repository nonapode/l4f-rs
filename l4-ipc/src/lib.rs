//! L4 IPC.
//!
//! This crate re-exports the mechanisms of `l4-ipc-core` and selects
//! protocols through cargo features.  `--features base` enables the
//! protocols every L4 system provides; single protocols
//! can be enabled by name, e.g. `--features proto-log`.
#![cfg_attr(not(test), no_std)]

pub use l4_ipc_core::*;

macro_rules! for_each_protocol {
    ( $macro:ident !) => {
        for_each_protocol!($macro ! ( ? ));
    };

    ($macro:ident ! ( $($prefix:expr, )* ? $(, $suffix:expr)* )) => {
        for_each_protocol!(Log => $macro ! ( $($prefix ,)* ? $(, $suffix)*));
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
}

#[cfg(feature = "base")]
pub mod base;
