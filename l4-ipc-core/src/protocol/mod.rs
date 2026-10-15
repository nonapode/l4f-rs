//! Protocol support: marshalling of requests and replies.

mod marshal;

pub use marshal::{Marshal, Reader, Unmarshal, Writer};
