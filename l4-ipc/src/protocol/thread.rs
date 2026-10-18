use l4_ipc_core::message::MapItem;
use l4_ipc_core::protocol::{Creatable, Marshal, Operation, Protocol, Reader, Unmarshal, Writer};
use l4_ipc_core::sys::{mword, umword};
use l4_ipc_core::types::fpage::{CapRights, Fpage, Object, Order, SndBase};
use l4_ipc_core::{CapIdx, Error};

/// The thread protocol marker (`L4_PROTO_THREAD`).
///
/// A thread is the unit of execution.  A thread capability is also named
/// where another protocol takes a receiver, for example as the thread an IPC
/// gate is bound to.
pub struct Thread;

impl Protocol for Thread {
    const PROTO: mword = -12;
}

/// A factory creates a thread (`l4_factory_create_thread`).
///
/// The request has no arguments beyond the type; the kernel stores the new
/// thread's capability in the slot of the [`Create`](super::factory::Create)
/// request.  The factory capability needs the send (`S`) right.
impl Creatable for Thread {
    type Handle = crate::kobject::Thread;
    type Args<'a> = ();
    type Reply = ();
}

const CONTROL: umword = 0;
const EX_REGS: umword = 1;
const SWITCH: umword = 2;
const STATS: umword = 3;
const VCPU_RESUME: umword = 4;
const REGISTER_DELETE_IRQ: umword = 5;
const MODIFY_SENDER: umword = 6;
const VCPU_CONTROL: umword = 7;
const VCPU_CONTROL_EXT: umword = VCPU_CONTROL | 0x10000;
const REGISTER_DOORBELL_IRQ: umword = 8;
const PF_TRAMP: umword = 9;

/// `L4_THREAD_CONTROL_SET_PAGER`: the pager is given.
const SET_PAGER: umword = 0x001_0000;
/// `L4_THREAD_CONTROL_BIND_TASK`: the task to bind the thread to is given.
const BIND_TASK: umword = 0x020_0000;
/// `L4_THREAD_CONTROL_ALIEN`: the alien state of the thread is set.
const ALIEN: umword = 0x040_0000;
/// `L4_THREAD_CONTROL_SET_EXC_HANDLER`: the exception handler is given.
const SET_EXC_HANDLER: umword = 0x100_0000;

/// `L4_THREAD_EX_REGS_CANCEL`: cancel an ongoing IPC in the thread.
pub const EX_REGS_CANCEL: umword = 0x1_0000;
/// `L4_THREAD_EX_REGS_TRIGGER_EXCEPTION`: raise an artificial exception.
pub const EX_REGS_TRIGGER_EXCEPTION: umword = 0x2_0000;

/// The task a thread is bound to, and the address of its UTCB.
///
/// The kernel interprets the task capability in the task the calling thread
/// executes in; the UTCB is mapped into the new thread's task.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Bind {
    /// Address of the new thread's UTCB.
    pub utcb: usize,
    /// The task to bind the thread to.
    pub task: CapIdx,
}

/// The parameters of [`Control`] (`l4_thread_control`).
///
/// The kernel applies only the parameters that are set; the sequence of
/// setters and the single commit of the C API is one request here.  Build it
/// with the methods and pass it to
/// [`Thread::control`](crate::extension::Thread::control).
///
/// ```no_run
/// # #[cfg(feature = "proto-thread")]
/// # fn control() -> Result<(), l4_ipc::Error> {
/// use l4_ipc::kobject;
/// use l4_ipc::prelude::*;
/// use l4_ipc::protocol::thread::ThreadControl;
/// use l4_ipc::CapIdx;
///
/// let worker = kobject::Thread::new(CapIdx::from_index(3));
/// worker.control(
///     ThreadControl::new()
///         .pager(CapIdx::from_index(4))
///         .exc_handler(CapIdx::from_index(5)),
/// )?;
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct ThreadControl {
    pager: Option<CapIdx>,
    exc_handler: Option<CapIdx>,
    alien: Option<bool>,
    bind: Option<Bind>,
}

impl ThreadControl {
    /// Control no parameter; only commits an empty control operation.
    pub const fn new() -> ThreadControl {
        ThreadControl {
            pager: None,
            exc_handler: None,
            alien: None,
            bind: None,
        }
    }

    /// Set the pager the thread sends page-fault IPCs to
    /// (`L4_THREAD_CONTROL_SET_PAGER`).
    pub const fn pager(mut self, pager: CapIdx) -> ThreadControl {
        self.pager = Some(pager);
        self
    }

    /// Set the exception handler of the thread
    /// (`L4_THREAD_CONTROL_SET_EXC_HANDLER`).
    pub const fn exc_handler(mut self, handler: CapIdx) -> ThreadControl {
        self.exc_handler = Some(handler);
        self
    }

    /// Set the alien state of the thread (`L4_THREAD_CONTROL_ALIEN`).
    pub const fn alien(mut self, on: bool) -> ThreadControl {
        self.alien = Some(on);
        self
    }

    /// Bind the thread to `task`, with its UTCB at the address of `utcb`
    /// (`L4_THREAD_CONTROL_BIND_TASK`).
    pub fn bind(mut self, utcb: &l4_ipc_core::sys::Utcb, task: CapIdx) -> ThreadControl {
        self.bind = Some(Bind {
            utcb: utcb as *const _ as usize,
            task,
        });
        self
    }
}

impl Marshal for ThreadControl {
    fn marshal(&self, w: &mut Writer<'_>) -> Result<(), Error> {
        let mut flags = CONTROL;
        flags |= if self.pager.is_some() { SET_PAGER } else { 0 };
        flags |= if self.bind.is_some() { BIND_TASK } else { 0 };
        flags |= if self.alien.is_some() { ALIEN } else { 0 };
        flags |= if self.exc_handler.is_some() {
            SET_EXC_HANDLER
        } else {
            0
        };

        w.push(flags)?;
        w.push(self.pager.map_or(0, |c| c.bits()))?;
        w.push(self.exc_handler.map_or(0, |c| c.bits()))?;
        w.push(0)?;
        w.push(if self.alien == Some(true) { ALIEN } else { 0 })?;
        w.push(self.bind.map_or(0, |b| b.utcb as umword))?;

        if let Some(bind) = self.bind {
            let task = Fpage::<Object>::new(bind.task, Order::ZERO, CapRights::RWS)?;
            MapItem::new(task, SndBase::default()).marshal(w)?;
        }
        Ok(())
    }
}

/// `l4_thread_control`: change parameters of a thread.
///
/// The flags of the parameters are part of the first word, so the opcode is
/// written by the request.  The task of a bind is a map item, which makes the
/// message carry one item.
pub struct Control;

impl Operation for Control {
    type Proto = Thread;
    const OPCODE: Option<umword> = None;
    type Request<'a> = ThreadControl;
    type Reply = ();
}

/// The request of [`ExRegs`]: the new instruction pointer, stack pointer and
/// the ex-regs flags (`L4_thread_ex_regs`).
///
/// `!0` leaves the instruction or stack pointer unchanged.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct ExRegsRequest {
    pub ip: umword,
    pub sp: umword,
    pub flags: umword,
}

impl Marshal for ExRegsRequest {
    fn marshal(&self, w: &mut Writer<'_>) -> Result<(), Error> {
        // The ex-regs flags share the first word with the opcode.
        w.push(EX_REGS | self.flags)?;
        w.push(self.ip)?;
        w.push(self.sp)
    }
}

/// The reply of [`ExRegs`]: the previous registers of the thread.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct PrevRegs {
    pub flags: umword,
    pub ip: umword,
    pub sp: umword,
}

impl Unmarshal for PrevRegs {
    fn unmarshal(r: &mut Reader<'_>) -> Result<Self, Error> {
        Ok(PrevRegs {
            flags: r.pop()?,
            ip: r.pop()?,
            sp: r.pop()?,
        })
    }
}

/// `l4_thread_ex_regs`: exchange the instruction and stack pointer of a
/// thread and return the previous values.
pub struct ExRegs;

impl Operation for ExRegs {
    type Proto = Thread;
    const OPCODE: Option<umword> = None;
    type Request<'a> = ExRegsRequest;
    type Reply = PrevRegs;
}

/// `l4_thread_switch`: switch to the thread.
pub struct Switch;

impl Operation for Switch {
    type Proto = Thread;
    const OPCODE: Option<umword> = Some(SWITCH);
    type Request<'a> = ();
    type Reply = ();
}

/// A time in microseconds (`l4_kernel_clock_t`).
///
/// It is a 64-bit value in a single message register; the targets of this
/// workspace are 64-bit, so it fits one word.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Micros(pub u64);

impl Micros {
    pub const fn get(self) -> u64 {
        self.0
    }
}

impl Unmarshal for Micros {
    fn unmarshal(r: &mut Reader<'_>) -> Result<Self, Error> {
        r.pop().map(|word| Micros(word as u64))
    }
}

/// `l4_thread_stats_time`: the consumed thread time in microseconds.
pub struct StatsTime;

impl Operation for StatsTime {
    type Proto = Thread;
    const OPCODE: Option<umword> = Some(STATS);
    type Request<'a> = ();
    type Reply = Micros;
}

/// `l4_thread_vcpu_resume`: resume a vCPU that was stopped for the calling
/// thread to handle its state.
pub struct VcpuResume;

impl Operation for VcpuResume {
    type Proto = Thread;
    const OPCODE: Option<umword> = Some(VCPU_RESUME);
    type Request<'a> = ();
    type Reply = ();
}

/// The request of [`RegisterDeleteIrq`] and [`RegisterDoorbellIrq`]: the IRQ
/// capability, as a map item with read, write and strong rights.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct IrqGate(pub CapIdx);

impl Marshal for IrqGate {
    fn marshal(&self, w: &mut Writer<'_>) -> Result<(), Error> {
        let irq = Fpage::<Object>::new(self.0, Order::ZERO, CapRights::RWS)?;
        MapItem::new(irq, SndBase::default()).marshal(w)
    }
}

/// `l4_thread_register_del_irq`: register an IRQ that is triggered when the
/// thread is deleted.
pub struct RegisterDeleteIrq;

impl Operation for RegisterDeleteIrq {
    type Proto = Thread;
    const OPCODE: Option<umword> = Some(REGISTER_DELETE_IRQ);
    type Request<'a> = IrqGate;
    type Reply = ();
}

/// `l4_thread_modify_sender`: change the sender IDs of all senders of a
/// thread that match a pattern.
///
/// Every entry compares a sender ID with `match_mask`; where it matches, the
/// bits in `del_bits` are cleared and the bits in `add_bits` are set.
pub struct ModifySender;

/// One pattern of [`ModifySender`].
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct ModifySenderEntry {
    /// The bits of the sender ID that take part in the comparison.
    pub match_mask: umword,
    /// The value the masked sender ID must equal.
    pub match_value: umword,
    /// The bits to clear where the sender matched.
    pub del_bits: umword,
    /// The bits to set where the sender matched.
    pub add_bits: umword,
}

/// The request of [`ModifySender`].
pub struct ModifySenderRequest<'a>(pub &'a [ModifySenderEntry]);

impl Marshal for ModifySenderRequest<'_> {
    fn marshal(&self, w: &mut Writer<'_>) -> Result<(), Error> {
        for entry in self.0 {
            w.push(entry.match_mask)?;
            w.push(entry.match_value)?;
            w.push(entry.del_bits)?;
            w.push(entry.add_bits)?;
        }
        Ok(())
    }
}

impl Operation for ModifySender {
    type Proto = Thread;
    const OPCODE: Option<umword> = Some(MODIFY_SENDER);
    type Request<'a> = ModifySenderRequest<'a>;
    type Reply = ();
}

/// `l4_thread_vcpu_control`: enable or disable the extended vCPU state of a
/// thread, with the address of the state-save area.
pub struct VcpuControl;

impl Operation for VcpuControl {
    type Proto = Thread;
    const OPCODE: Option<umword> = Some(VCPU_CONTROL);
    type Request<'a> = umword;
    type Reply = ();
}

/// `l4_thread_vcpu_control_ext`: like [`VcpuControl`], for the extended
/// state-save area.
pub struct VcpuControlExt;

impl Operation for VcpuControlExt {
    type Proto = Thread;
    const OPCODE: Option<umword> = Some(VCPU_CONTROL_EXT);
    type Request<'a> = umword;
    type Reply = ();
}

/// `l4_thread_register_doorbell_irq`: register the direct IRQ injection
/// doorbell IRQ of a thread.
pub struct RegisterDoorbellIrq;

impl Operation for RegisterDoorbellIrq {
    type Proto = Thread;
    const OPCODE: Option<umword> = Some(REGISTER_DOORBELL_IRQ);
    type Request<'a> = IrqGate;
    type Reply = ();
}

const PF_TRAMP_SETUP: umword = PF_TRAMP;
const PF_TRAMP_RESUME: umword = PF_TRAMP | 0x10000;
const PF_TRAMP_REFLECT: umword = PF_TRAMP | 0x20000;
const PF_TRAMP_DISABLE: umword = PF_TRAMP | 0x30000;
const PF_TRAMP_ENABLE: umword = PF_TRAMP | 0x40000;

/// `l4_thread_pf_trampoline_setup`: install the page-fault trampoline of the
/// thread, with the address of its state.
pub struct PfTrampSetup;

impl Operation for PfTrampSetup {
    type Proto = Thread;
    const OPCODE: Option<umword> = Some(PF_TRAMP_SETUP);
    type Request<'a> = umword;
    type Reply = ();
}

/// `l4_thread_pf_trampoline_resume`: resume the thread from the page-fault
/// trampoline to userland.
pub struct PfTrampResume;

impl Operation for PfTrampResume {
    type Proto = Thread;
    const OPCODE: Option<umword> = Some(PF_TRAMP_RESUME);
    type Request<'a> = ();
    type Reply = ();
}

/// `l4_thread_pf_trampoline_reflect`: resume the thread from the page-fault
/// trampoline by raising an exception.
pub struct PfTrampReflect;

impl Operation for PfTrampReflect {
    type Proto = Thread;
    const OPCODE: Option<umword> = Some(PF_TRAMP_REFLECT);
    type Request<'a> = ();
    type Reply = ();
}

/// The reply of [`PfTrampDisable`]: whether the trampoline was enabled.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct WasEnabled(pub bool);

impl Unmarshal for WasEnabled {
    fn unmarshal(r: &mut Reader<'_>) -> Result<Self, Error> {
        r.pop().map(|word| WasEnabled(word != 0))
    }
}

/// `l4_thread_pf_trampoline_disable`: disable the page-fault trampoline of
/// the thread and say whether it was enabled.
pub struct PfTrampDisable;

impl Operation for PfTrampDisable {
    type Proto = Thread;
    const OPCODE: Option<umword> = Some(PF_TRAMP_DISABLE);
    type Request<'a> = ();
    type Reply = WasEnabled;
}

/// `l4_thread_pf_trampoline_enable`: re-enable a disabled page-fault
/// trampoline.
pub struct PfTrampEnable;

impl Operation for PfTrampEnable {
    type Proto = Thread;
    const OPCODE: Option<umword> = Some(PF_TRAMP_ENABLE);
    type Request<'a> = ();
    type Reply = ();
}

#[cfg(all(test, not(target_os = "none")))]
mod tests {
    use super::*;
    use l4_ipc_core::message::{Tag, TagFlags};
    use l4_ipc_core::protocol::encode_request;
    use l4_ipc_core::sys::Utcb;

    fn regs<const N: usize>(utcb: &Utcb) -> [umword; N] {
        core::array::from_fn(|i| utcb.mr(i))
    }

    fn task(index: usize) -> umword {
        index << 12 | 3 << 4 | 7
    }

    #[test]
    fn protocol_value_matches_the_kernel() {
        assert_eq!(Thread::PROTO, -12);
    }

    #[test]
    fn an_empty_control_sends_six_zero_words() {
        let utcb = Utcb::zeroed();
        let tag = encode_request::<Control>(&utcb, &ThreadControl::new()).unwrap();

        assert_eq!(
            (tag.label(), tag.words(), tag.items(), tag.flags()),
            (-12, 6, 0, TagFlags::NONE)
        );
        assert_eq!(regs::<6>(&utcb), [0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn control_sets_the_flags_of_the_parameters() {
        let utcb = Utcb::zeroed();
        let request = ThreadControl::new()
            .pager(CapIdx::from_index(4))
            .exc_handler(CapIdx::from_index(5))
            .alien(true);
        let tag = encode_request::<Control>(&utcb, &request).unwrap();

        assert_eq!((tag.words(), tag.items()), (6, 0));
        assert_eq!(
            regs::<6>(&utcb),
            [
                SET_PAGER | SET_EXC_HANDLER | ALIEN,
                4 << 12,
                5 << 12,
                0,
                ALIEN,
                0
            ]
        );
    }

    #[test]
    fn control_alien_off_does_not_set_the_value() {
        let utcb = Utcb::zeroed();
        let request = ThreadControl::new().alien(false);
        encode_request::<Control>(&utcb, &request).unwrap();
        assert_eq!(utcb.mr(0), ALIEN);
        assert_eq!(utcb.mr(4), 0);
    }

    #[test]
    fn binding_a_thread_sends_the_utcb_and_the_task_as_an_item() {
        let utcb = Utcb::zeroed();
        let request = ThreadControl::new().bind(&utcb, CapIdx::from_index(3));
        let tag = encode_request::<Control>(&utcb, &request).unwrap();

        assert_eq!((tag.words(), tag.items()), (6, 1));
        assert_eq!(utcb.mr(0), BIND_TASK);
        assert_eq!(utcb.mr(5), &utcb as *const _ as umword);
        assert_eq!(utcb.mr(6), 8);
        assert_eq!(utcb.mr(7), task(3));
    }

    #[test]
    fn ex_regs_shares_the_first_word_with_the_flags() {
        let utcb = Utcb::zeroed();
        let request = ExRegsRequest {
            ip: 0x1000,
            sp: 0x8000,
            flags: EX_REGS_CANCEL | EX_REGS_TRIGGER_EXCEPTION,
        };
        let tag = encode_request::<ExRegs>(&utcb, &request).unwrap();

        assert_eq!((tag.label(), tag.words()), (-12, 3));
        assert_eq!(
            regs::<3>(&utcb),
            [EX_REGS | 0x30000, 0x1000, 0x8000]
        );
    }

    #[test]
    fn ex_regs_reply_is_the_previous_registers() {
        let utcb = Utcb::zeroed();
        utcb.set_mr(0, 0x11);
        utcb.set_mr(1, 0x1000);
        utcb.set_mr(2, 0x8000);

        let reply = PrevRegs::unmarshal(&mut Reader::new(&utcb, Tag::new(0, 3, 0, TagFlags::NONE)));
        assert_eq!(
            reply,
            Ok(PrevRegs {
                flags: 0x11,
                ip: 0x1000,
                sp: 0x8000
            })
        );
    }

    #[test]
    fn the_simple_operations_are_one_word() {
        for (opcode, label) in [
            (SWITCH, <Switch as Operation>::OPCODE),
            (STATS, <StatsTime as Operation>::OPCODE),
            (VCPU_RESUME, <VcpuResume as Operation>::OPCODE),
            (
                PF_TRAMP_RESUME,
                <PfTrampResume as Operation>::OPCODE,
            ),
            (PF_TRAMP_REFLECT, <PfTrampReflect as Operation>::OPCODE),
            (PF_TRAMP_ENABLE, <PfTrampEnable as Operation>::OPCODE),
        ] {
            assert_eq!(label, Some(opcode));
        }
    }

    #[test]
    fn switch_is_a_single_word() {
        let utcb = Utcb::zeroed();
        let tag = encode_request::<Switch>(&utcb, &()).unwrap();
        assert_eq!((tag.label(), tag.words(), tag.items()), (-12, 1, 0));
        assert_eq!(utcb.mr(0), SWITCH);
    }

    #[test]
    fn stats_time_reply_is_a_64_bit_value() {
        let utcb = Utcb::zeroed();
        utcb.set_mr(0, 0x1234_5678);

        let time = Micros::unmarshal(&mut Reader::new(&utcb, Tag::new(0, 1, 0, TagFlags::NONE)));
        assert_eq!(time, Ok(Micros(0x1234_5678)));
    }

    #[test]
    fn registering_an_irq_sends_it_as_an_item() {
        let utcb = Utcb::zeroed();
        let tag = encode_request::<RegisterDeleteIrq>(&utcb, &IrqGate(CapIdx::from_index(6))).unwrap();

        assert_eq!((tag.words(), tag.items()), (1, 1));
        assert_eq!(regs::<3>(&utcb), [REGISTER_DELETE_IRQ, 8, task(6)]);

        let utcb = Utcb::zeroed();
        let tag =
            encode_request::<RegisterDoorbellIrq>(&utcb, &IrqGate(CapIdx::from_index(7))).unwrap();
        assert_eq!((tag.words(), tag.items()), (1, 1));
        assert_eq!(regs::<3>(&utcb), [REGISTER_DOORBELL_IRQ, 8, task(7)]);
    }

    #[test]
    fn modify_sender_sends_four_words_per_entry() {
        let utcb = Utcb::zeroed();
        let entries = [
            ModifySenderEntry {
                match_mask: 0xff,
                match_value: 0x10,
                del_bits: 1,
                add_bits: 2,
            },
            ModifySenderEntry {
                match_mask: 0xf0,
                match_value: 0x20,
                del_bits: 3,
                add_bits: 4,
            },
        ];
        let tag = encode_request::<ModifySender>(&utcb, &ModifySenderRequest(&entries)).unwrap();

        assert_eq!((tag.words(), tag.items()), (1 + 8, 0));
        assert_eq!(
            regs::<9>(&utcb),
            [MODIFY_SENDER, 0xff, 0x10, 1, 2, 0xf0, 0x20, 3, 4]
        );
    }

    #[test]
    fn vcpu_control_sends_the_state() {
        let utcb = Utcb::zeroed();
        let tag = encode_request::<VcpuControl>(&utcb, &0x2000).unwrap();
        assert_eq!((tag.words(), tag.items()), (2, 0));
        assert_eq!(regs::<2>(&utcb), [VCPU_CONTROL, 0x2000]);
    }

    #[test]
    fn vcpu_control_ext_uses_the_extended_opcode() {
        let utcb = Utcb::zeroed();
        let tag = encode_request::<VcpuControlExt>(&utcb, &0x2000).unwrap();
        assert_eq!((tag.words(), tag.items()), (2, 0));
        assert_eq!(regs::<2>(&utcb), [VCPU_CONTROL_EXT, 0x2000]);
    }

    #[test]
    fn pf_trampoline_setup_sends_the_state() {
        let utcb = Utcb::zeroed();
        let tag = encode_request::<PfTrampSetup>(&utcb, &0x4000).unwrap();
        assert_eq!(tag.words(), 2);
        assert_eq!(regs::<2>(&utcb), [PF_TRAMP_SETUP, 0x4000]);
    }

    #[test]
    fn pf_trampoline_disable_reports_whether_it_was_enabled() {
        let utcb = Utcb::zeroed();
        let tag = encode_request::<PfTrampDisable>(&utcb, &()).unwrap();
        assert_eq!(tag.words(), 1);
        assert_eq!(regs::<1>(&utcb), [PF_TRAMP_DISABLE]);

        utcb.set_mr(0, 1);
        let enabled = WasEnabled::unmarshal(&mut Reader::new(&utcb, Tag::new(0, 1, 0, TagFlags::NONE)));
        assert_eq!(enabled, Ok(WasEnabled(true)));
    }
}
