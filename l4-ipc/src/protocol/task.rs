use core::marker::PhantomData;
use l4_ipc_core::types::fpage::{Any, CapRights, Fpage, Memory, Obj, Order, SndBase};
use l4_ipc_core::message::MapItem;
use l4_ipc_core::protocol::{
    Creatable, Flexpage, Marshal, Operation, Protocol, Reader, Unmarshal, Writer,
};
use l4_ipc_core::sys::{mword, umword};
use l4_ipc_core::{CapIdx, Error};

/// The task protocol marker (`L4_PROTO_TASK`).
///
/// A task is an address space with a capability space.  The task protocol
/// maps and unmaps flexpages of memory, I/O ports and capabilities.
pub struct Task;

impl Protocol for Task {
    const PROTO: mword = -11;
}

/// What an unmap removes (`l4_unmap_flags_t`).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum UnmapScope {
    /// The mappings derived from the flexpage in other spaces, but not the
    /// flexpage of the task itself (`L4_FP_OTHER_SPACES`).
    OtherSpaces,
    /// The flexpage in all spaces, including the task itself
    /// (`L4_FP_ALL_SPACES`).
    AllSpaces,
    /// Like [`AllSpaces`](UnmapScope::AllSpaces), and the kernel object a
    /// capability refers to is deleted (`L4_FP_DELETE_OBJ`).
    DeleteObj,
}

impl UnmapScope {
    /// The value of the map mask.  The flags are in the upper half of the
    /// low 32 bits whatever the width of a word.
    pub const fn bits(self) -> umword {
        match self {
            UnmapScope::OtherSpaces => 0,
            UnmapScope::AllSpaces => 0x8000_0000,
            UnmapScope::DeleteObj => 0xc000_0000,
        }
    }
}

/// The memory flexpage for the UTCB area of a new task, the argument of
/// creating a task.  The factory returns the flexpage it used:
///
/// ```no_run
/// # #[cfg(feature = "proto-factory")]
/// # fn create() -> Result<(), l4_ipc::Error> {
/// use l4_ipc::types::fpage::{Fpage, MemRights, Memory, Order};
/// use l4_ipc::prelude::*;
/// use l4_ipc::protocol::{self, task::UtcbArea};
/// use l4_ipc::{kobject, CapIdx};
///
/// let factory = kobject::Factory::new(CapIdx::from_index(2));
/// let area = Fpage::<Memory>::new(0xb000_0000, Order::PAGE, MemRights::RW)?;
/// let (task, used): (kobject::Task, _) =
///     factory.create_returning::<protocol::Task>(CapIdx::from_index(0x21), UtcbArea(area))?;
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct UtcbArea(pub Fpage<Memory>);

impl Marshal for UtcbArea {
    fn marshal(&self, w: &mut Writer<'_>) -> Result<(), Error> {
        Flexpage(self.0).marshal(w)
    }
}

/// A factory creates a task from the flexpage for its UTCB area
/// (`l4_factory_create_task`) and replies with the flexpage it used.
impl Creatable for Task {
    type Handle = crate::kobject::Task;
    type Args<'a> = UtcbArea;
    type Reply = Fpage<Any>;
}

const MAP: umword = 0;
const UNMAP: umword = 1;
const CAP_INFO: umword = 2;
const ADD_KU_MEM: umword = 3;

/// `l4_task_map`: map or grant a flexpage of the task `src` to the task this
/// is invoked on.
///
/// The request carries the send base and the flexpage as words, and the
/// capability of the source task as a map item with read, write and strong
/// rights (an object flexpage of order 0).  The source task has to be a valid
/// capability.
pub struct Map<K>(PhantomData<fn() -> K>);

/// The request of [`Map`].
pub struct MapRequest<K> {
    /// The task whose flexpage is mapped.
    pub src: CapIdx,
    /// The flexpage in the space of `src`.
    pub fpage: Fpage<K>,
    /// Where the flexpage is placed in the receiving task, and how.
    pub base: SndBase,
}

impl<K> Marshal for MapRequest<K> {
    fn marshal(&self, w: &mut Writer<'_>) -> Result<(), Error> {
        let src = Fpage::<Obj>::new(self.src, Order::ZERO, CapRights::RWS)?;

        w.push(self.base.bits())?;
        w.push(self.fpage.bits())?;
        MapItem::new(src, SndBase::default()).marshal(w)
    }
}

impl<K> Operation for Map<K> {
    type Proto = Task;
    const OPCODE: Option<umword> = Some(MAP);
    type Request<'a> = MapRequest<K>;
    type Reply = ();
}

/// `l4_task_unmap`: revoke flexpages.  The rights of a flexpage are the
/// rights that are revoked.
pub struct Unmap<K>(PhantomData<fn() -> K>);

/// The request of [`Unmap`].
pub struct UnmapRequest<'a, K> {
    /// What is removed.
    pub scope: UnmapScope,
    /// The flexpages to unmap; as many as fit into the message registers.
    pub fpages: &'a [Fpage<K>],
}

impl<K> Marshal for UnmapRequest<'_, K> {
    fn marshal(&self, w: &mut Writer<'_>) -> Result<(), Error> {
        w.push(self.scope.bits())?;
        self.fpages
            .iter()
            .try_for_each(|fpage| w.push(fpage.bits()))
    }
}

/// The kinds of flexpages are all `'static`; the bound lets the request borrow
/// a slice of flexpages.
impl<K: 'static> Operation for Unmap<K> {
    type Proto = Task;
    const OPCODE: Option<umword> = Some(UNMAP);
    type Request<'a> = UnmapRequest<'a, K>;
    type Reply = ();
}

/// `l4_task_cap_valid`: whether a capability slot of the task holds a
/// capability.
///
/// The operation code is shared with [`CapEqual`]; the kernel tells them
/// apart by the number of words.
pub struct CapValid;

/// The request of [`CapValid`].
pub struct ValidRequest(pub CapIdx);

impl Marshal for ValidRequest {
    fn marshal(&self, w: &mut Writer<'_>) -> Result<(), Error> {
        // The lowest bit of the selector is no part of the slot.
        w.push(self.0.bits() & !1)
    }
}

/// The reply of [`CapValid`]: whether the slot holds a capability.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Present(pub bool);

impl Unmarshal for Present {
    fn unmarshal(r: &mut Reader<'_>) -> Result<Self, Error> {
        // The answer is the label of the reply: positive if present.
        Ok(Present(r.label() > 0))
    }
}

impl Operation for CapValid {
    type Proto = Task;
    const OPCODE: Option<umword> = Some(CAP_INFO);
    type Request<'a> = ValidRequest;
    type Reply = Present;
}

/// `l4_task_cap_equal`: whether two capability slots of the task refer to
/// the same object.
pub struct CapEqual;

/// The request of [`CapEqual`].
pub struct EqualRequest {
    pub a: CapIdx,
    pub b: CapIdx,
}

impl Marshal for EqualRequest {
    fn marshal(&self, w: &mut Writer<'_>) -> Result<(), Error> {
        w.push(self.a.bits())?;
        w.push(self.b.bits())
    }
}

/// The reply of [`CapEqual`]: whether both refer to the same object.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Same(pub bool);

impl Unmarshal for Same {
    fn unmarshal(r: &mut Reader<'_>) -> Result<Self, Error> {
        // The answer is the label of the reply: 1 if equal.
        Ok(Same(r.label() == 1))
    }
}

impl Operation for CapEqual {
    type Proto = Task;
    const OPCODE: Option<umword> = Some(CAP_INFO);
    type Request<'a> = EqualRequest;
    type Reply = Same;
}

/// `l4_task_add_ku_mem`: add kernel-user memory to the task, in the
/// flexpage.  The kernel replies with the flexpage it used.  The task
/// capability needs the right to add kernel-user memory, else the kernel
/// replies with `-L4_ENOSYS`.
pub struct AddKuMem;

impl Operation for AddKuMem {
    type Proto = Task;
    const OPCODE: Option<umword> = Some(ADD_KU_MEM);
    type Request<'a> = Fpage<Memory>;
    type Reply = Fpage<Any>;
}

#[cfg(all(test, not(target_os = "none")))]
mod tests {
    use super::*;
    use core::mem::size_of;
    use l4_ipc_core::types::fpage::{MemRights, Order, Transfer};
    use l4_ipc_core::message::{Tag, TagFlags};
    use l4_ipc_core::protocol::encode_request;
    use l4_ipc_core::sys::Utcb;

    fn regs<const N: usize>(utcb: &Utcb) -> [umword; N] {
        core::array::from_fn(|i| utcb.mr(i))
    }

    fn memory(addr: umword, order: u8) -> Fpage<Memory> {
        Fpage::<Memory>::new(addr, Order::new(order).unwrap(), MemRights::RW).unwrap()
    }

    #[test]
    fn protocol_value_matches_the_kernel() {
        assert_eq!(Task::PROTO, -11);
    }

    #[cfg(feature = "proto-factory")]
    #[test]
    fn creating_a_task_sends_the_utcb_area_as_a_flexpage_argument() {
        use crate::protocol::factory::{Create, CreateRequest};

        let utcb = Utcb::zeroed();
        let area = memory(0xb000_0000, 16);
        let request = CreateRequest::<Task> {
            slot: CapIdx::from_index(0x20),
            args: UtcbArea(area),
        };
        let tag = encode_request::<Create<Task>>(&utcb, &request).unwrap();

        // l4_factory_create_task: the type, then the argument with its
        // header (L4_VARG_TYPE_FPAGE, one word).
        assert_eq!((tag.label(), tag.words(), tag.items()), (-15, 3, 0));
        assert_eq!(
            regs::<3>(&utcb),
            [
                -11isize as umword,
                0x3 | (size_of::<umword>() as umword) << 16,
                area.bits()
            ]
        );
    }

    #[test]
    fn unmap_flags_match_the_abi() {
        assert_eq!(UnmapScope::OtherSpaces.bits(), 0);
        assert_eq!(UnmapScope::AllSpaces.bits(), 0x8000_0000);
        assert_eq!(UnmapScope::DeleteObj.bits(), 0xc000_0000);
    }

    #[test]
    fn map_sends_the_fpage_and_the_source_task_as_an_item() {
        let utcb = Utcb::zeroed();
        let fpage = memory(0x1000, 12);
        let base = SndBase::new(0x4000).unwrap().with_transfer(Transfer::Grant);
        let request = MapRequest {
            src: CapIdx::from_index(5),
            fpage,
            base,
        };
        let tag = encode_request::<Map<Memory>>(&utcb, &request).unwrap();

        // l4_task_map: op, snd_base, snd_fpage, then the map item with
        // l4_map_obj_control(0, 0) and l4_obj_fpage(src_task, 0, RWS).
        assert_eq!(
            (tag.label(), tag.words(), tag.items(), tag.flags()),
            (-11, 3, 1, TagFlags::NONE)
        );
        assert_eq!(
            regs::<5>(&utcb),
            [0, 0x4002, fpage.bits(), 8, 5 << 12 | 3 << 4 | 7]
        );
    }

    #[test]
    fn map_needs_a_valid_source_task() {
        let utcb = Utcb::zeroed();
        let request = MapRequest {
            src: CapIdx::INVALID,
            fpage: memory(0x1000, 12),
            base: SndBase::default(),
        };

        assert_eq!(
            encode_request::<Map<Memory>>(&utcb, &request),
            Err(Error::Fpage(l4_ipc_core::types::fpage::Error::InvalidCap))
        );
    }

    #[test]
    fn unmap_sends_the_flags_and_every_fpage() {
        let utcb = Utcb::zeroed();
        let fpages = [memory(0x1000, 12), memory(0x4000, 14)];
        let request = UnmapRequest {
            scope: UnmapScope::AllSpaces,
            fpages: &fpages,
        };
        let tag = encode_request::<Unmap<Memory>>(&utcb, &request).unwrap();

        assert_eq!((tag.label(), tag.words(), tag.items()), (-11, 4, 0));
        assert_eq!(
            regs::<4>(&utcb),
            [1, 0x8000_0000, fpages[0].bits(), fpages[1].bits()]
        );
    }

    #[test]
    fn cap_valid_is_two_words_without_the_low_bit() {
        let utcb = Utcb::zeroed();
        let selector = CapIdx::from_bits(CapIdx::from_index(9).bits() | 1);
        let tag = encode_request::<CapValid>(&utcb, &ValidRequest(selector)).unwrap();

        assert_eq!((tag.label(), tag.words()), (-11, 2));
        assert_eq!(regs::<2>(&utcb), [2, 9 << 12]);
    }

    #[test]
    fn cap_equal_is_three_words() {
        let utcb = Utcb::zeroed();
        let request = EqualRequest {
            a: CapIdx::from_index(3),
            b: CapIdx::from_index(4),
        };
        let tag = encode_request::<CapEqual>(&utcb, &request).unwrap();

        assert_eq!((tag.label(), tag.words()), (-11, 3));
        assert_eq!(regs::<3>(&utcb), [2, 3 << 12, 4 << 12]);
    }

    #[test]
    fn add_ku_mem_sends_one_flexpage() {
        let utcb = Utcb::zeroed();
        let fpage = memory(0x10000, 16);
        let tag = encode_request::<AddKuMem>(&utcb, &fpage).unwrap();

        assert_eq!((tag.label(), tag.words()), (-11, 2));
        assert_eq!(regs::<2>(&utcb), [3, fpage.bits()]);
    }

    #[test]
    fn replies_are_read_from_the_label() {
        let utcb = Utcb::zeroed();
        let reply = |label| Reader::new(&utcb, Tag::new(label, 0, 0, TagFlags::NONE));

        assert_eq!(Present::unmarshal(&mut reply(1)), Ok(Present(true)));
        assert_eq!(Present::unmarshal(&mut reply(0)), Ok(Present(false)));
        assert_eq!(Same::unmarshal(&mut reply(1)), Ok(Same(true)));
        assert_eq!(Same::unmarshal(&mut reply(0)), Ok(Same(false)));
    }

    #[test]
    fn the_used_flexpage_comes_back() {
        let utcb = Utcb::zeroed();
        utcb.set_mr(0, 0x10316);

        let tag = Tag::new(0, 1, 0, TagFlags::NONE);
        let used = Fpage::<Any>::unmarshal(&mut Reader::new(&utcb, tag)).unwrap();

        assert_eq!(used.bits(), 0x10316);
        assert_eq!(size_of::<Fpage<Any>>(), size_of::<umword>());
    }
}
