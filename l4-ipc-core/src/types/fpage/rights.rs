//! The rights fields of the flexpage kinds.
use super::flags::flags;

flags! {
    /// The rights of a memory flexpage (`L4_FPAGE_X`, `L4_FPAGE_W` and
    /// `L4_FPAGE_RO`).
    MemRights {
        /// Executable.
        X = 1,
        /// Writable.
        W = 2,
        /// Readable.
        R = 4,
    }
}

impl MemRights {
    /// Read-write.
    pub const RW: MemRights = MemRights::R.union(MemRights::W);
    /// Read-execute.
    pub const RX: MemRights = MemRights::R.union(MemRights::X);
    /// Read-write-execute.
    pub const RWX: MemRights = MemRights::RW.union(MemRights::X);
}

flags! {
    /// The rights of an object flexpage, i.e. of a capability
    /// (`L4_CAP_FPAGE_*`).
    CapRights {
        /// Write: the capability can be used to modify the object.
        W = 1,
        /// Strong: the capability can be used to send to the object.
        S = 2,
        /// Read: the capability can be used to read the object.
        R = 4,
        /// Delete: the object can be deleted through the capability.
        D = 8,
    }
}

impl CapRights {
    /// Read-write.
    pub const RW: CapRights = CapRights::R.union(CapRights::W);
    /// Read-strong.
    pub const RS: CapRights = CapRights::R.union(CapRights::S);
    /// Read-write-strong.
    pub const RWS: CapRights = CapRights::RW.union(CapRights::S);
    /// Read-write-strong-delete.
    pub const RWSD: CapRights = CapRights::RWS.union(CapRights::D);
    /// Read-write-delete.
    pub const RWD: CapRights = CapRights::RW.union(CapRights::D);
    /// Read-strong-delete.
    pub const RSD: CapRights = CapRights::RS.union(CapRights::D);
}

/// The rights of the special flexpages: none.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default, Debug)]
pub struct NoRights;

#[cfg(all(test, not(target_os = "none")))]
mod tests {
    use super::*;
    use arbitrary_int::u4;

    #[test]
    fn memory_rights_match_the_abi() {
        assert_eq!(MemRights::X.bits(), u4::new(1));
        assert_eq!(MemRights::W.bits(), u4::new(2));
        assert_eq!(MemRights::R.bits(), u4::new(4));
        assert_eq!(MemRights::RW.bits(), u4::new(6));
        assert_eq!(MemRights::RX.bits(), u4::new(5));
        assert_eq!(MemRights::RWX.bits(), u4::new(7));
    }

    #[test]
    fn capability_rights_match_the_abi() {
        assert_eq!(CapRights::W.bits(), u4::new(1));
        assert_eq!(CapRights::S.bits(), u4::new(2));
        assert_eq!(CapRights::R.bits(), u4::new(4));
        assert_eq!(CapRights::D.bits(), u4::new(8));
        assert_eq!(CapRights::RW.bits(), u4::new(5));
        assert_eq!(CapRights::RS.bits(), u4::new(6));
        assert_eq!(CapRights::RWS.bits(), u4::new(7));
        assert_eq!(CapRights::RWSD.bits(), u4::new(15));
        assert_eq!(CapRights::RWD.bits(), u4::new(13));
        assert_eq!(CapRights::RSD.bits(), u4::new(14));
    }

    #[test]
    fn the_two_kinds_do_not_mix() {
        // Both have a write right, but it is not the same bit.
        assert_ne!(MemRights::W.bits(), CapRights::W.bits());
    }

    #[test]
    fn rights_combine() {
        let mut rights = CapRights::R;
        rights |= CapRights::S;
        assert_eq!(rights, CapRights::RS);
        assert_eq!(CapRights::R | CapRights::W, CapRights::RW);
        assert!(CapRights::RWS.contains(CapRights::RS));
        assert!(!CapRights::RS.contains(CapRights::W));
        assert_eq!(CapRights::default(), CapRights::NONE);
    }

    #[test]
    fn raw_bits_are_kept() {
        assert_eq!(CapRights::from_bits(u4::new(0xf)), CapRights::RWSD);
        assert_eq!(CapRights::from_bits(u4::new(0)).bits(), u4::new(0));
        for bits in 0..16 {
            let rights = MemRights::from_bits(u4::new(bits));
            assert_eq!(MemRights::from_bits(rights.bits()), rights);
        }
    }

    #[test]
    fn debug_names_the_rights() {
        extern crate std;
        use std::format;

        assert_eq!(format!("{:?}", MemRights::RW), "MemRights(W|R)");
        assert_eq!(format!("{:?}", CapRights::NONE), "CapRights(NONE)");
        assert_eq!(
            format!("{:?}", MemRights::from_bits(u4::new(8))),
            "MemRights(0x8)"
        );
    }
}
