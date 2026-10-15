//! L4 IPC.
//!
//! This crate re-exports the mechanisms of `l4-ipc-core` and selects
//! protocol crates (`l4-proto-*`) through cargo features.  `--features
//! base` enables the protocols every L4 system provides; single protocols
//! can be enabled by name, e.g. `--features log`.
#![cfg_attr(not(test), no_std)]

pub use l4_ipc_core::*;

macro_rules! for_each_protocol {
    ( $macro:ident ) => {
        $crate::for_each_protocol!($macro =>
            //irq => Irq,
            //semaphore => Semaphore,
            Log,
            //ipc => Ipc,
        );
    };

    ( $macro:ident => $( $snake:ident ),* $(,)? ) => {
        $(
            paste::paste! {
                $macro!(#[cfg(feature = "proto-" $snake:snake ) ]
                        [<l4_proto_ $snake:snake >]
                        [< $snake:snake >]
                        [< $snake:camel >]
                );
            }
        )*
    };
}

pub(crate) use for_each_protocol;

/// prelude module.
pub mod prelude {
    macro_rules! prelude {
        (#[$meta:meta] $root:ident $snake:ident $camel:ident ) => {
            #[$meta]
            pub use $root::Extension as _; // bring extension in scope
        };
    }
    crate::for_each_protocol!(prelude);
}

/// IPC extensions.
pub mod extension {
    #[allow(unused)]
    macro_rules! extension {
        (#[$meta:meta] $root:ident $snake:ident $camel:ident) => {
            #[$meta]
            pub use $root::Extension as $camel;

            #[$meta]
            pub use $root::extension as $snake;
        };
    }
    super::for_each_protocol!(extension);
}

/// IPC protocols.
pub mod protocol {
    #[allow(unused)]
    macro_rules! protocol {
        (#[$meta:meta] $root:ident $snake:ident $camel:ident ) => {
            #[$meta]
            pub use $root::Protocol as $camel;

            #[$meta]
            pub use $root::protocol as $snake;
        };
    }

    super::for_each_protocol!(protocol);
}
