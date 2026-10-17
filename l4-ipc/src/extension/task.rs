use l4_ipc_core::types::fpage::{CapRights, Fpage, Generic, Kind, Memory, Object, Order, SndBase};
use l4_ipc_core::protocol::Speaks;
use l4_ipc_core::sys::Utcb;
use l4_ipc_core::{CapIdx, Error, Timeout};

use crate::protocol::task::{
    AddKuMem, CapEqual, CapValid, EqualRequest, Map, MapRequest, Task as Protocol, Unmap,
    UnmapRequest, UnmapScope, ValidRequest,
};

/// Convenience methods for handles that speak the task protocol.
///
/// The task a method is called on is the one whose address space or
/// capability space is changed.  Every method has a `_with` variant that
/// takes the UTCB.
///
/// ```no_run
/// use l4_ipc::types::fpage::{Fpage, MemRights, Memory, Order};
/// use l4_ipc::prelude::*;
/// use l4_ipc::protocol::{task::UnmapScope, Task};
/// use l4_ipc_core::handle;
///
/// handle!(Domain: Task);
///
/// fn revoke(task: Domain) -> Result<(), l4_ipc::Error> {
///     let page = Fpage::<Memory>::new(0x1000, Order::PAGE, MemRights::RW)?;
///     task.unmap(&[page], UnmapScope::OtherSpaces)
/// }
/// ```
///
/// Only handles that speak the task protocol are tasks:
///
/// ```compile_fail,E0599
/// use l4_ipc::prelude::*;
/// use l4_ipc::CapIdx;
/// use l4_ipc_core::handle;
/// use l4_ipc_core::protocol::Protocol;
///
/// struct Other;
/// impl Protocol for Other { const PROTO: isize = -99; }
/// handle!(Console: Other);
///
/// // Console does not speak the task protocol.
/// fn ask(console: Console) {
///     let _ = console.cap_valid(CapIdx::from_index(0x20));
/// }
/// ```
pub trait Task: Speaks<Protocol> {
    /// Map or grant `fpage` of the task `src` into this task, at `base`
    /// (`l4_task_map`).
    fn map_with<K: Kind, S>(
        &self,
        utcb: &Utcb,
        src: &S,
        fpage: Fpage<K>,
        base: SndBase,
    ) -> Result<(), Error>
    where
        S: Speaks<Protocol>,
    {
        let request = MapRequest {
            src: src.idx(),
            fpage,
            base,
        };
        self.invoke_with::<Map<K>>(utcb, request, Timeout::NEVER)
    }

    /// See [`map_with`](Task::map_with).
    fn map<K: Kind, S>(&self, src: &S, fpage: Fpage<K>, base: SndBase) -> Result<(), Error>
    where
        S: Speaks<Protocol>,
    {
        self.map_with(Utcb::current(), src, fpage, base)
    }

    /// Revoke the flexpages from this task and the tasks it mapped them to
    /// (`l4_task_unmap`).  They have to fit into the message registers.
    fn unmap_with<K: Kind + 'static>(
        &self,
        utcb: &Utcb,
        fpages: &[Fpage<K>],
        scope: UnmapScope,
    ) -> Result<(), Error> {
        self.invoke_with::<Unmap<K>>(utcb, UnmapRequest { scope, fpages }, Timeout::NEVER)
    }

    /// See [`unmap_with`](Task::unmap_with).
    fn unmap<K: Kind + 'static>(&self, fpages: &[Fpage<K>], scope: UnmapScope) -> Result<(), Error> {
        self.unmap_with(Utcb::current(), fpages, scope)
    }

    /// Whether the capability slot holds a capability (`l4_task_cap_valid`).
    fn cap_valid_with(&self, utcb: &Utcb, cap: CapIdx) -> Result<bool, Error> {
        let present = self.invoke_with::<CapValid>(utcb, ValidRequest(cap), Timeout::NEVER)?;
        Ok(present.0)
    }

    /// See [`cap_valid_with`](Task::cap_valid_with).
    fn cap_valid(&self, cap: CapIdx) -> Result<bool, Error> {
        self.cap_valid_with(Utcb::current(), cap)
    }

    /// Whether two capability slots refer to the same object
    /// (`l4_task_cap_equal`).
    fn cap_equal_with(&self, utcb: &Utcb, a: CapIdx, b: CapIdx) -> Result<bool, Error> {
        let same = self.invoke_with::<CapEqual>(utcb, EqualRequest { a, b }, Timeout::NEVER)?;
        Ok(same.0)
    }

    /// See [`cap_equal_with`](Task::cap_equal_with).
    fn cap_equal(&self, a: CapIdx, b: CapIdx) -> Result<bool, Error> {
        self.cap_equal_with(Utcb::current(), a, b)
    }

    /// Add kernel-user memory to the task and return the flexpage the
    /// kernel used (`l4_task_add_ku_mem`).
    fn add_ku_mem_with(&self, utcb: &Utcb, fpage: Fpage<Memory>) -> Result<Generic, Error> {
        self.invoke_with::<AddKuMem>(utcb, fpage, Timeout::NEVER)
    }

    /// See [`add_ku_mem_with`](Task::add_ku_mem_with).
    fn add_ku_mem(&self, fpage: Fpage<Memory>) -> Result<Generic, Error> {
        self.add_ku_mem_with(Utcb::current(), fpage)
    }

    /// Delete the object the capability refers to, and unmap the capability
    /// from all spaces (`l4_task_delete_obj`).  Needs the delete right.
    fn delete_obj_with(&self, utcb: &Utcb, obj: CapIdx) -> Result<(), Error> {
        let fpage = Fpage::<Object>::new(obj, Order::ZERO, CapRights::RWSD)?;
        self.unmap_with(utcb, &[fpage], UnmapScope::DeleteObj)
    }

    /// See [`delete_obj_with`](Task::delete_obj_with).
    fn delete_obj(&self, obj: CapIdx) -> Result<(), Error> {
        self.delete_obj_with(Utcb::current(), obj)
    }

    /// Unmap the capability from all spaces, without deleting the object
    /// (`l4_task_release_cap`).
    fn release_cap_with(&self, utcb: &Utcb, cap: CapIdx) -> Result<(), Error> {
        let fpage = Fpage::<Object>::new(cap, Order::ZERO, CapRights::RWSD)?;
        self.unmap_with(utcb, &[fpage], UnmapScope::AllSpaces)
    }

    /// See [`release_cap_with`](Task::release_cap_with).
    fn release_cap(&self, cap: CapIdx) -> Result<(), Error> {
        self.release_cap_with(Utcb::current(), cap)
    }
}

impl<H: Speaks<Protocol>> Task for H {}

#[cfg(all(test, not(target_os = "none")))]
mod tests {
    use super::*;
    use l4_ipc_core::handle;

    handle!(Domain: Protocol);

    /// The methods are only type-checked, calling them needs a kernel.
    #[allow(dead_code)]
    fn typed(task: Domain, other: Domain, page: Fpage<Memory>) -> Result<(), Error> {
        task.map(&other, page, SndBase::default())?;
        task.unmap(&[page], UnmapScope::OtherSpaces)?;
        let _: bool = task.cap_valid(CapIdx::from_index(1))?;
        let _: bool = task.cap_equal(CapIdx::from_index(1), CapIdx::from_index(2))?;
        let _: Generic = task.add_ku_mem(page)?;
        task.delete_obj(CapIdx::from_index(3))?;
        task.release_cap(CapIdx::from_index(3))
    }

    #[test]
    fn every_handle_that_speaks_the_protocol_is_a_task() {
        fn needs_task<T: Task>(t: T) -> CapIdx {
            t.idx()
        }
        let idx = CapIdx::from_index(1);
        assert_eq!(needs_task(Domain::new(idx)), idx);
    }
}
