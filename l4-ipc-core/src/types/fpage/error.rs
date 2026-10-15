/// Why a flexpage cannot be built.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Error {
    /// The order is too small or too big for the kind of flexpage.
    Order,
    /// The address, capability slot or port is not a multiple of the size
    /// of the flexpage.
    Misaligned,
    /// The capability selector is invalid.
    InvalidCap,
}
