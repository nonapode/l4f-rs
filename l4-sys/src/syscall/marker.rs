/// Capability token proving that code runs inside `l4-sys`.
///
/// The type is public so that other crates can *name* it in their
/// implementations of [`Input`](super::Input) and
/// [`Output`](super::Output), but only this crate can construct a value of
/// it.  Therefore only `l4-sys` itself can encode or decode register words.
#[derive(Clone, Copy)]
pub struct Marker(pub(super) ());
