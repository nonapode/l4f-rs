use l4_ipc_core::protocol::Speaks;
use l4_ipc_core::sys::Utcb;
use l4_ipc_core::{CapIdx, Error, Timeout};

use crate::protocol::thread::{
    Control, ExRegs, ExRegsRequest, IrqGate, Micros, ModifySender, ModifySenderEntry,
    ModifySenderRequest, PfTrampDisable, PfTrampEnable, PfTrampReflect, PfTrampResume, PfTrampSetup,
    PrevRegs, RegisterDeleteIrq, RegisterDoorbellIrq, StatsTime, Switch, Thread as Protocol,
    ThreadControl, VcpuControl, VcpuControlExt, VcpuResume, WasEnabled,
};

/// Convenience methods for handles that speak the thread protocol.
///
/// Every method has a `_with` variant that takes the UTCB.
///
/// ```no_run
/// use l4_ipc::kobject;
/// use l4_ipc::prelude::*;
/// use l4_ipc::protocol::thread::ThreadControl;
/// use l4_ipc::CapIdx;
///
/// fn start(thread: kobject::Thread) -> Result<(), l4_ipc::Error> {
///     thread.control(ThreadControl::new().pager(CapIdx::from_index(4)))?;
///     let _prev = thread.ex_regs(0x1000, 0x8000, 0)?;
///     Ok(())
/// }
/// ```
///
/// Only handles that speak the thread protocol are threads:
///
/// ```compile_fail,E0599
/// use l4_ipc::protocol::thread::ThreadControl;
/// use l4_ipc::prelude::*;
/// use l4_ipc_core::handle;
/// use l4_ipc_core::protocol::Protocol;
///
/// struct Other;
/// impl Protocol for Other { const PROTO: isize = -99; }
/// handle!(Console: Other);
///
/// // Console does not speak the thread protocol.
/// fn ask(console: Console) {
///     let _ = console.control(ThreadControl::new());
/// }
/// ```
pub trait Thread: Speaks<Protocol> {
    /// Change the parameters of the thread (`l4_thread_control`), using the
    /// given UTCB.
    fn control_with(&self, utcb: &Utcb, control: ThreadControl) -> Result<(), Error> {
        self.invoke_with::<Control>(utcb, control, Timeout::NEVER)
    }

    /// See [`control_with`](Thread::control_with).
    fn control(&self, control: ThreadControl) -> Result<(), Error> {
        self.control_with(Utcb::current(), control)
    }

    /// Exchange the instruction and stack pointer of the thread and return
    /// the previous registers (`l4_thread_ex_regs`), using the given UTCB.
    fn ex_regs_with(
        &self,
        utcb: &Utcb,
        ip: usize,
        sp: usize,
        flags: usize,
    ) -> Result<PrevRegs, Error> {
        let request = ExRegsRequest {
            ip: ip as _,
            sp: sp as _,
            flags: flags as _,
        };
        self.invoke_with::<ExRegs>(utcb, request, Timeout::NEVER)
    }

    /// See [`ex_regs_with`](Thread::ex_regs_with).
    fn ex_regs(&self, ip: usize, sp: usize, flags: usize) -> Result<PrevRegs, Error> {
        self.ex_regs_with(Utcb::current(), ip, sp, flags)
    }

    /// Switch to the thread (`l4_thread_switch`), using the given UTCB.
    fn switch_with(&self, utcb: &Utcb) -> Result<(), Error> {
        self.invoke_with::<Switch>(utcb, (), Timeout::NEVER)
    }

    /// See [`switch_with`](Thread::switch_with).
    fn switch(&self) -> Result<(), Error> {
        self.switch_with(Utcb::current())
    }

    /// The consumed thread time in microseconds (`l4_thread_stats_time`),
    /// using the given UTCB.
    fn stats_time_with(&self, utcb: &Utcb) -> Result<Micros, Error> {
        self.invoke_with::<StatsTime>(utcb, (), Timeout::NEVER)
    }

    /// See [`stats_time_with`](Thread::stats_time_with).
    fn stats_time(&self) -> Result<Micros, Error> {
        self.stats_time_with(Utcb::current())
    }

    /// Resume a vCPU stopped for this thread (`l4_thread_vcpu_resume`),
    /// using the given UTCB.
    fn vcpu_resume_with(&self, utcb: &Utcb) -> Result<(), Error> {
        self.invoke_with::<VcpuResume>(utcb, (), Timeout::NEVER)
    }

    /// See [`vcpu_resume_with`](Thread::vcpu_resume_with).
    fn vcpu_resume(&self) -> Result<(), Error> {
        self.vcpu_resume_with(Utcb::current())
    }

    /// Register the IRQ that is triggered when the thread is deleted
    /// (`l4_thread_register_del_irq`), using the given UTCB.
    fn register_delete_irq_with(&self, utcb: &Utcb, irq: CapIdx) -> Result<(), Error> {
        self.invoke_with::<RegisterDeleteIrq>(utcb, IrqGate(irq), Timeout::NEVER)
    }

    /// See [`register_delete_irq_with`](Thread::register_delete_irq_with).
    fn register_delete_irq(&self, irq: CapIdx) -> Result<(), Error> {
        self.register_delete_irq_with(Utcb::current(), irq)
    }

    /// Register the direct IRQ injection doorbell IRQ of the thread
    /// (`l4_thread_register_doorbell_irq`), using the given UTCB.
    fn register_doorbell_irq_with(&self, utcb: &Utcb, irq: CapIdx) -> Result<(), Error> {
        self.invoke_with::<RegisterDoorbellIrq>(utcb, IrqGate(irq), Timeout::NEVER)
    }

    /// See [`register_doorbell_irq_with`](Thread::register_doorbell_irq_with).
    fn register_doorbell_irq(&self, irq: CapIdx) -> Result<(), Error> {
        self.register_doorbell_irq_with(Utcb::current(), irq)
    }

    /// Change the sender IDs of all senders that match one of `entries`
    /// (`l4_thread_modify_sender`), using the given UTCB.
    fn modify_sender_with(
        &self,
        utcb: &Utcb,
        entries: &[ModifySenderEntry],
    ) -> Result<(), Error> {
        self.invoke_with::<ModifySender>(utcb, ModifySenderRequest(entries), Timeout::NEVER)
    }

    /// See [`modify_sender_with`](Thread::modify_sender_with).
    fn modify_sender(&self, entries: &[ModifySenderEntry]) -> Result<(), Error> {
        self.modify_sender_with(Utcb::current(), entries)
    }

    /// Enable or disable the vCPU state of the thread, with the address of
    /// its state-save area (`l4_thread_vcpu_control`), using the given UTCB.
    fn vcpu_control_with(&self, utcb: &Utcb, state: usize) -> Result<(), Error> {
        self.invoke_with::<VcpuControl>(utcb, state as _, Timeout::NEVER)
    }

    /// See [`vcpu_control_with`](Thread::vcpu_control_with).
    fn vcpu_control(&self, state: usize) -> Result<(), Error> {
        self.vcpu_control_with(Utcb::current(), state)
    }

    /// Like [`vcpu_control_with`](Thread::vcpu_control_with), for the
    /// extended state-save area (`l4_thread_vcpu_control_ext`).
    fn vcpu_control_ext_with(&self, utcb: &Utcb, state: usize) -> Result<(), Error> {
        self.invoke_with::<VcpuControlExt>(utcb, state as _, Timeout::NEVER)
    }

    /// See [`vcpu_control_ext_with`](Thread::vcpu_control_ext_with).
    fn vcpu_control_ext(&self, state: usize) -> Result<(), Error> {
        self.vcpu_control_ext_with(Utcb::current(), state)
    }

    /// Install the page-fault trampoline of the thread with the address of
    /// its state (`l4_thread_pf_trampoline_setup`), using the given UTCB.
    fn pf_trampoline_setup_with(&self, utcb: &Utcb, state: usize) -> Result<(), Error> {
        self.invoke_with::<PfTrampSetup>(utcb, state as _, Timeout::NEVER)
    }

    /// See [`pf_trampoline_setup_with`](Thread::pf_trampoline_setup_with).
    fn pf_trampoline_setup(&self, state: usize) -> Result<(), Error> {
        self.pf_trampoline_setup_with(Utcb::current(), state)
    }

    /// Resume the thread from the page-fault trampoline to userland
    /// (`l4_thread_pf_trampoline_resume`), using the given UTCB.
    fn pf_trampoline_resume_with(&self, utcb: &Utcb) -> Result<(), Error> {
        self.invoke_with::<PfTrampResume>(utcb, (), Timeout::NEVER)
    }

    /// See [`pf_trampoline_resume_with`](Thread::pf_trampoline_resume_with).
    fn pf_trampoline_resume(&self) -> Result<(), Error> {
        self.pf_trampoline_resume_with(Utcb::current())
    }

    /// Resume the thread from the page-fault trampoline by raising an
    /// exception (`l4_thread_pf_trampoline_reflect`), using the given UTCB.
    fn pf_trampoline_reflect_with(&self, utcb: &Utcb) -> Result<(), Error> {
        self.invoke_with::<PfTrampReflect>(utcb, (), Timeout::NEVER)
    }

    /// See [`pf_trampoline_reflect_with`](Thread::pf_trampoline_reflect_with).
    fn pf_trampoline_reflect(&self) -> Result<(), Error> {
        self.pf_trampoline_reflect_with(Utcb::current())
    }

    /// Disable the page-fault trampoline of the thread and say whether it
    /// was enabled (`l4_thread_pf_trampoline_disable`), using the given UTCB.
    fn pf_trampoline_disable_with(&self, utcb: &Utcb) -> Result<WasEnabled, Error> {
        self.invoke_with::<PfTrampDisable>(utcb, (), Timeout::NEVER)
    }

    /// See [`pf_trampoline_disable_with`](Thread::pf_trampoline_disable_with).
    fn pf_trampoline_disable(&self) -> Result<WasEnabled, Error> {
        self.pf_trampoline_disable_with(Utcb::current())
    }

    /// Re-enable a disabled page-fault trampoline
    /// (`l4_thread_pf_trampoline_enable`), using the given UTCB.
    fn pf_trampoline_enable_with(&self, utcb: &Utcb) -> Result<(), Error> {
        self.invoke_with::<PfTrampEnable>(utcb, (), Timeout::NEVER)
    }

    /// See [`pf_trampoline_enable_with`](Thread::pf_trampoline_enable_with).
    fn pf_trampoline_enable(&self) -> Result<(), Error> {
        self.pf_trampoline_enable_with(Utcb::current())
    }
}

impl<H: Speaks<Protocol>> Thread for H {}

#[cfg(all(test, not(target_os = "none")))]
mod tests {
    use super::*;
    use l4_ipc_core::handle;

    handle!(Worker: Protocol);

    /// The methods are only type-checked, calling them needs a kernel.
    #[allow(dead_code)]
    fn typed(worker: Worker) -> Result<(), Error> {
        use crate::protocol::thread::{ModifySenderEntry, EX_REGS_CANCEL};

        worker.control(ThreadControl::new().alien(true))?;
        let _: PrevRegs = worker.ex_regs(0x1000, 0x8000, EX_REGS_CANCEL)?;
        worker.switch()?;
        let _: Micros = worker.stats_time()?;
        worker.vcpu_resume()?;
        worker.register_delete_irq(CapIdx::from_index(6))?;
        worker.register_doorbell_irq(CapIdx::from_index(7))?;
        worker.modify_sender(&[ModifySenderEntry::default()])?;
        worker.vcpu_control(0x2000)?;
        worker.vcpu_control_ext(0x2000)?;
        worker.pf_trampoline_setup(0x4000)?;
        worker.pf_trampoline_resume()?;
        worker.pf_trampoline_reflect()?;
        let _: WasEnabled = worker.pf_trampoline_disable()?;
        worker.pf_trampoline_enable()
    }

    #[test]
    fn every_handle_that_speaks_the_protocol_is_a_thread() {
        fn needs_thread<T: Thread>(t: T) -> CapIdx {
            t.idx()
        }
        let idx = CapIdx::from_index(3);
        assert_eq!(needs_thread(Worker::new(idx)), idx);
    }
}
