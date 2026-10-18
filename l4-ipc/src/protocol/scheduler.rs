use l4_ipc_core::message::MapItem;
use l4_ipc_core::protocol::{Marshal, Operation, Protocol, Reader, Unmarshal, Writer};
use l4_ipc_core::sys::{mword, umword};
use l4_ipc_core::types::fpage::{CapRights, Fpage, Object, Order, SndBase};
use l4_ipc_core::{CapIdx, Error};

/// The scheduler protocol marker (`L4_PROTO_SCHEDULER`).
///
/// A scheduler manages the CPU resources of the system.  It is provided by
/// the kernel, not created with a factory.
pub struct Scheduler;

impl Protocol for Scheduler {
    const PROTO: mword = -14;
}

/// A set of CPUs (`l4_sched_cpu_set_t`).
///
/// `gran_offset` packs the number of CPUs each bit of `map` describes
/// (granularity, in log2) into the high 8 bits and the number of the first
/// CPU (offset) into the low 24 bits.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct CpuSet {
    /// Granularity in the high 8 bits, offset in the low 24 bits.
    pub gran_offset: umword,
    /// Bitmap of CPUs, one bit per `2^granularity` CPUs.
    pub map: umword,
}

impl CpuSet {
    /// Build a CPU set.  `offset` must be a multiple of `2^granularity`.
    pub const fn new(offset: umword, granularity: u8, map: umword) -> CpuSet {
        CpuSet {
            gran_offset: ((granularity as umword) << 24) | (offset & 0x00ff_ffff),
            map,
        }
    }

    /// The number of CPUs each bit of the map describes, in log2.
    pub const fn granularity(self) -> u8 {
        (self.gran_offset >> 24) as u8
    }

    /// The number of the first CPU the map describes.
    pub const fn offset(self) -> umword {
        self.gran_offset & 0x00ff_ffff
    }
}

/// The scheduling classes a scheduler offers (`L4_scheduler_classes`).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct SchedulerClasses(pub umword);

impl SchedulerClasses {
    /// The fixed-priority scheduler (`L4_SCHEDULER_CLASS_FIXED_PRIO`).
    pub const FIXED_PRIO: umword = 1 << 1;
    /// The weighted fair queuing scheduler (`L4_SCHEDULER_CLASS_WFQ`).
    pub const WFQ: umword = 1 << 2;

    /// Whether the scheduler offers `class`.
    pub const fn contains(self, class: umword) -> bool {
        self.0 & class != 0
    }
}

/// The parameters of [`RunThread`] (`l4_sched_param_t`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct SchedParam {
    /// CPU affinity of the thread.
    pub affinity: CpuSet,
    /// Priority, 1..255; 0 is reserved for the kernel.
    pub prio: umword,
    /// Timeslice in microseconds.
    pub quantum: umword,
}

impl SchedParam {
    /// A parameter with the given priority and timeslice, affine to all
    /// CPUs.
    pub const fn new(prio: umword, quantum: umword) -> SchedParam {
        SchedParam {
            affinity: CpuSet::new(0, u8::MAX, 1),
            prio,
            quantum,
        }
    }
}

const INFO: umword = 0;
const RUN_THREAD: umword = 1;
const IDLE_TIME: umword = 2;

/// The request of [`Info`]: the CPU set of interest.  Only its
/// `gran_offset` is part of the request.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct InfoRequest(pub CpuSet);

impl Marshal for InfoRequest {
    fn marshal(&self, w: &mut Writer<'_>) -> Result<(), Error> {
        w.push(self.0.gran_offset)
    }
}

/// The reply of [`Info`].
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct SchedulerInfo {
    /// Bitmap of the online CPUs at `offset`.
    pub map: umword,
    /// The maximum number of CPUs ever available.
    pub cpu_max: umword,
    /// The scheduling classes the scheduler offers.
    pub sched_classes: SchedulerClasses,
}

impl Unmarshal for SchedulerInfo {
    fn unmarshal(r: &mut Reader<'_>) -> Result<Self, Error> {
        Ok(SchedulerInfo {
            map: r.pop()?,
            cpu_max: r.pop()?,
            sched_classes: SchedulerClasses(r.pop()?),
        })
    }
}

/// `l4_scheduler_info`: query the online CPUs, the maximum number of CPUs
/// and the scheduling classes of the scheduler.
pub struct Info;

impl Operation for Info {
    type Proto = Scheduler;
    const OPCODE: Option<umword> = Some(INFO);
    type Request<'a> = InfoRequest;
    type Reply = SchedulerInfo;
}

/// The request of [`RunThread`]: the parameters and the thread to run.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct RunThreadRequest {
    pub param: SchedParam,
    pub thread: CapIdx,
}

impl Marshal for RunThreadRequest {
    fn marshal(&self, w: &mut Writer<'_>) -> Result<(), Error> {
        w.push(self.param.affinity.gran_offset)?;
        w.push(self.param.affinity.map)?;
        w.push(self.param.prio)?;
        w.push(self.param.quantum)?;

        let thread = Fpage::<Object>::new(self.thread, Order::ZERO, CapRights::RWS)?;
        MapItem::new(thread, SndBase::default()).marshal(w)
    }
}

/// `l4_scheduler_run_thread`: start the thread on this scheduler.  The
/// thread is passed as a map item.
pub struct RunThread;

impl Operation for RunThread {
    type Proto = Scheduler;
    const OPCODE: Option<umword> = Some(RUN_THREAD);
    type Request<'a> = RunThreadRequest;
    type Reply = ();
}

/// A time in microseconds (`l4_kernel_clock_t`).  The targets of this
/// workspace are 64-bit, so it fits one message register.
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

/// The request of [`IdleTime`]: the CPU set whose idle time is asked for.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct IdleTimeRequest(pub CpuSet);

impl Marshal for IdleTimeRequest {
    fn marshal(&self, w: &mut Writer<'_>) -> Result<(), Error> {
        w.push(self.0.gran_offset)?;
        w.push(self.0.map)
    }
}

/// `l4_scheduler_idle_time`: the idle time of the CPUs in the set, in
/// microseconds.
pub struct IdleTime;

impl Operation for IdleTime {
    type Proto = Scheduler;
    const OPCODE: Option<umword> = Some(IDLE_TIME);
    type Request<'a> = IdleTimeRequest;
    type Reply = Micros;
}

#[cfg(all(test, not(target_os = "none")))]
mod tests {
    use super::*;
    use l4_ipc_core::message::{Tag, TagFlags};
    use l4_ipc_core::protocol::encode_request;
    use l4_ipc_core::sys::Utcb;

    fn thread(index: usize) -> umword {
        index << 12 | 3 << 4 | 7
    }

    #[test]
    fn protocol_value_matches_the_kernel() {
        assert_eq!(Scheduler::PROTO, -14);
    }

    #[test]
    fn a_cpu_set_packs_granularity_and_offset() {
        let set = CpuSet::new(4, 2, 0b1010);
        assert_eq!(set.granularity(), 2);
        assert_eq!(set.offset(), 4);
        assert_eq!(set.gran_offset, (2 << 24) | 4);
        assert_eq!(set.map, 0b1010);
    }

    #[test]
    fn a_sched_param_is_affine_to_all_cpus() {
        let param = SchedParam::new(0x10, 1000);
        assert_eq!(param.affinity, CpuSet::new(0, u8::MAX, 1));
        assert_eq!(param.prio, 0x10);
        assert_eq!(param.quantum, 1000);
    }

    #[test]
    fn classes_are_a_bitmap() {
        let classes = SchedulerClasses(
            SchedulerClasses::FIXED_PRIO | SchedulerClasses::WFQ,
        );
        assert!(classes.contains(SchedulerClasses::FIXED_PRIO));
        assert!(classes.contains(SchedulerClasses::WFQ));
        assert!(!SchedulerClasses(1).contains(SchedulerClasses::WFQ));
    }

    #[test]
    fn info_sends_only_the_granularity_and_offset() {
        let utcb = Utcb::zeroed();
        let tag = encode_request::<Info>(&utcb, &InfoRequest(CpuSet::new(4, 2, 0xff))).unwrap();

        assert_eq!(
            (tag.label(), tag.words(), tag.items(), tag.flags()),
            (-14, 2, 0, TagFlags::NONE)
        );
        assert_eq!([utcb.mr(0), utcb.mr(1)], [INFO, (2 << 24) | 4]);
    }

    #[test]
    fn info_reply_carries_map_max_and_classes() {
        let utcb = Utcb::zeroed();
        utcb.set_mr(0, 0b11);
        utcb.set_mr(1, 8);
        utcb.set_mr(2, SchedulerClasses::FIXED_PRIO);

        let info = SchedulerInfo::unmarshal(&mut Reader::new(&utcb, Tag::new(0, 3, 0, TagFlags::NONE)));
        assert_eq!(
            info,
            Ok(SchedulerInfo {
                map: 0b11,
                cpu_max: 8,
                sched_classes: SchedulerClasses(SchedulerClasses::FIXED_PRIO),
            })
        );
    }

    #[test]
    fn run_thread_sends_the_parameters_and_the_thread_as_an_item() {
        let utcb = Utcb::zeroed();
        let request = RunThreadRequest {
            param: SchedParam {
                affinity: CpuSet::new(2, 1, 0b100),
                prio: 0x20,
                quantum: 5000,
            },
            thread: CapIdx::from_index(3),
        };
        let tag = encode_request::<RunThread>(&utcb, &request).unwrap();

        assert_eq!((tag.words(), tag.items()), (5, 1));
        assert_eq!(
            [
                utcb.mr(0),
                utcb.mr(1),
                utcb.mr(2),
                utcb.mr(3),
                utcb.mr(4),
                utcb.mr(5),
                utcb.mr(6)
            ],
            [RUN_THREAD, (1 << 24) | 2, 0b100, 0x20, 5000, 8, thread(3)]
        );
    }

    #[test]
    fn idle_time_sends_the_cpu_set_and_returns_micros() {
        let utcb = Utcb::zeroed();
        let tag = encode_request::<IdleTime>(&utcb, &IdleTimeRequest(CpuSet::new(0, 0, 0b1))).unwrap();
        assert_eq!((tag.words(), tag.items()), (3, 0));
        assert_eq!([utcb.mr(0), utcb.mr(1), utcb.mr(2)], [IDLE_TIME, 0, 0b1]);

        utcb.set_mr(0, 42);
        let micros = Micros::unmarshal(&mut Reader::new(&utcb, Tag::new(0, 1, 0, TagFlags::NONE)));
        assert_eq!(micros, Ok(Micros(42)));
    }
}
