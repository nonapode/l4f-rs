use l4_ipc_core::protocol::Speaks;
use l4_ipc_core::sys::Utcb;
use l4_ipc_core::{Error, Timeout};

use crate::protocol::scheduler::{
    CpuSet, IdleTime, IdleTimeRequest, Info, InfoRequest, Micros, RunThread, RunThreadRequest,
    SchedParam, Scheduler as Protocol, SchedulerInfo,
};
use crate::protocol::thread::Thread as ThreadProto;

/// Convenience methods for handles that speak the scheduler protocol.
///
/// Every method has a `_with` variant that takes the UTCB.
///
/// ```no_run
/// use l4_ipc::kobject;
/// use l4_ipc::prelude::*;
/// use l4_ipc::protocol::scheduler::{CpuSet, SchedParam};
/// use l4_ipc::CapIdx;
///
/// fn run(scheduler: kobject::Scheduler, thread: kobject::Thread) -> Result<(), l4_ipc::Error> {
///     let online = scheduler.is_online(0)?;
///     if online {
///         scheduler.run_thread(&thread, SchedParam::new(0x10, 1000))?;
///     }
///     Ok(())
/// }
/// ```
///
/// Only handles that speak the scheduler protocol are schedulers:
///
/// ```compile_fail,E0599
/// use l4_ipc::prelude::*;
/// use l4_ipc::protocol::scheduler::CpuSet;
/// use l4_ipc_core::handle;
/// use l4_ipc_core::protocol::Protocol;
///
/// struct Other;
/// impl Protocol for Other { const PROTO: isize = -99; }
/// handle!(Thing: Other);
///
/// // Thing is not a scheduler.
/// fn ask(thing: Thing) {
///     let _ = thing.info(CpuSet::new(0, 0, 1));
/// }
/// ```
pub trait Scheduler: Speaks<Protocol> {
    /// Query the online CPUs, the maximum number of CPUs and the scheduling
    /// classes for the CPUs at the offset of `cpus`
    /// (`l4_scheduler_info`), using the given UTCB.
    fn info_with(&self, utcb: &Utcb, cpus: CpuSet) -> Result<SchedulerInfo, Error> {
        self.invoke_with::<Info>(utcb, InfoRequest(cpus), Timeout::NEVER)
    }

    /// See [`info_with`](Scheduler::info_with).
    fn info(&self, cpus: CpuSet) -> Result<SchedulerInfo, Error> {
        self.info_with(Utcb::current(), cpus)
    }

    /// Start `thread` on this scheduler with `param`
    /// (`l4_scheduler_run_thread`), using the given UTCB.
    fn run_thread_with<T>(&self, utcb: &Utcb, thread: &T, param: SchedParam) -> Result<(), Error>
    where
        T: Speaks<ThreadProto>,
    {
        let request = RunThreadRequest {
            param,
            thread: thread.idx(),
        };
        self.invoke_with::<RunThread>(utcb, request, Timeout::NEVER)
    }

    /// See [`run_thread_with`](Scheduler::run_thread_with).
    fn run_thread<T>(&self, thread: &T, param: SchedParam) -> Result<(), Error>
    where
        T: Speaks<ThreadProto>,
    {
        self.run_thread_with(Utcb::current(), thread, param)
    }

    /// The idle time of the CPUs in `cpus`, in microseconds
    /// (`l4_scheduler_idle_time`), using the given UTCB.
    fn idle_time_with(&self, utcb: &Utcb, cpus: CpuSet) -> Result<Micros, Error> {
        self.invoke_with::<IdleTime>(utcb, IdleTimeRequest(cpus), Timeout::NEVER)
    }

    /// See [`idle_time_with`](Scheduler::idle_time_with).
    fn idle_time(&self, cpus: CpuSet) -> Result<Micros, Error> {
        self.idle_time_with(Utcb::current(), cpus)
    }

    /// Whether the CPU is online (`l4_scheduler_is_online`), using the given
    /// UTCB.
    fn is_online_with(&self, utcb: &Utcb, cpu: usize) -> Result<bool, Error> {
        let info = self.info_with(utcb, CpuSet::new(cpu as _, 0, 1))?;
        Ok(info.map & 1 != 0)
    }

    /// See [`is_online_with`](Scheduler::is_online_with).
    fn is_online(&self, cpu: usize) -> Result<bool, Error> {
        self.is_online_with(Utcb::current(), cpu)
    }
}

impl<H: Speaks<Protocol>> Scheduler for H {}

#[cfg(all(test, not(target_os = "none")))]
mod tests {
    use super::*;
    use crate::kobject;
    use l4_ipc_core::handle;
    use l4_ipc_core::CapIdx;

    handle!(Sched: Protocol);

    /// The methods are only type-checked, calling them needs a kernel.
    #[allow(dead_code)]
    fn typed(sched: Sched, thread: kobject::Thread) -> Result<(), Error> {
        let _: SchedulerInfo = sched.info(CpuSet::new(0, 0, 1))?;
        sched.run_thread(&thread, SchedParam::new(0x10, 1000))?;
        let _: Micros = sched.idle_time(CpuSet::new(0, 0, 1))?;
        let _: bool = sched.is_online(0)?;
        Ok(())
    }

    #[test]
    fn every_handle_that_speaks_the_protocol_is_a_scheduler() {
        fn needs_scheduler<S: Scheduler>(s: S) -> CapIdx {
            s.idx()
        }
        let idx = CapIdx::from_index(7);
        assert_eq!(needs_scheduler(Sched::new(idx)), idx);
    }
}
