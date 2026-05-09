//! Durable planning payload owned by the planning capability.
//!
//! The planner stays deterministic: it derives an ordered task graph from the
//! objective packet, records dependencies and lineage, and exposes the same
//! kernel-visible `TaskReady` evidence used by the runtime.

use crate::capability::{EvidenceProducer, EvidenceSubmission, PacketEffect};
use crate::kernel::{Evidence, GateId, Packet, mix};

const PLAN_SCHEMA_VERSION: u64 = 2;
const MAX_PLAN_TASKS: u8 = 8;
pub const PLAN_RECEIPT_SCHEMA_VERSION: u64 = 1;
pub const PLAN_RECEIPT_RECORD: u64 = 0x91a0_0001;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlanDecision {
    Ready,
    Blocked,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlanTask {
    pub task_id: u64,
    pub depends_on: u64,
    pub ordinal: u8,
    pub ready: bool,
}

impl PlanTask {
    pub fn is_valid(self, objective_id: u64) -> bool {
        self.task_id != 0
            && self.ordinal != 0
            && self.task_id / 100 == objective_id
            && (self.depends_on == 0 || self.depends_on / 100 == objective_id)
    }

    fn hash(self) -> u64 {
        let mut h = 0x5441_534b_504c_414eu64;
        h = mix(h, self.task_id);
        h = mix(h, self.depends_on);
        h = mix(h, u64::from(self.ordinal));
        h = mix(h, u64::from(self.ready));
        h.max(1)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlanRecord {
    pub objective_id: u64,
    pub task_id: u64,
    pub ready_tasks: u8,
    pub dependency_hash: u64,
    pub objective_hash: u64,
    pub plan_version: u64,
    pub plan_revision: u64,
    pub task_count: u8,
    pub completed_tasks: u8,
    pub ready_set_hash: u64,
    pub lineage_hash: u64,
    pub tasks: Vec<PlanTask>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlanReceipt {
    pub schema_version: u64,
    pub record_type: u64,
    pub objective_id: u64,
    pub task_id: u64,
    pub task_count: u64,
    pub completed_tasks: u64,
    pub ready_tasks: u64,
    pub plan_version: u64,
    pub plan_revision: u64,
    pub objective_hash: u64,
    pub dependency_hash: u64,
    pub ready_set_hash: u64,
    pub lineage_hash: u64,
    pub payload_hash: u64,
    pub verdict: PlanDecision,
    pub receipt_hash: u64,
}

impl PlanRecord {
    pub fn from_packet(packet: Packet) -> Self {
        let objective_id = packet.objective_id;
        let required = packet.objective_required_tasks.clamp(1, MAX_PLAN_TASKS);
        let completed = packet.objective_done_tasks.min(required);
        let tasks = plan_tasks(objective_id, required, completed);
        let ready_tasks = tasks
            .iter()
            .filter(|task| task.ready)
            .count()
            .min(u8::MAX as usize) as u8;
        let task_id = tasks
            .iter()
            .find(|task| task.ready)
            .map(|task| task.task_id)
            .unwrap_or(0);
        let objective_hash = objective_hash(packet);
        let dependency_hash = dependency_hash(&tasks);
        let ready_set_hash = ready_set_hash(&tasks);
        let plan_revision = packet.revision.max(1);
        let lineage_hash = lineage_hash(
            objective_hash,
            dependency_hash,
            ready_set_hash,
            plan_revision,
            packet.objective_done_tasks,
        );

        Self {
            objective_id,
            task_id,
            ready_tasks,
            dependency_hash,
            objective_hash,
            plan_version: PLAN_SCHEMA_VERSION,
            plan_revision,
            task_count: tasks.len().min(u8::MAX as usize) as u8,
            completed_tasks: completed,
            ready_set_hash,
            lineage_hash,
            tasks,
        }
    }

    pub fn decision(&self) -> PlanDecision {
        if self.is_valid() && self.ready_tasks != 0 {
            PlanDecision::Ready
        } else {
            PlanDecision::Blocked
        }
    }

    pub fn is_valid(&self) -> bool {
        self.objective_id != 0
            && self.plan_version == PLAN_SCHEMA_VERSION
            && self.plan_revision != 0
            && self.task_count != 0
            && self.task_count as usize == self.tasks.len()
            && self.completed_tasks <= self.task_count
            && self.task_id != 0
            && self.ready_tasks != 0
            && self.dependency_hash == dependency_hash(&self.tasks)
            && self.ready_set_hash == ready_set_hash(&self.tasks)
            && self.objective_hash != 0
            && self.lineage_hash
                == lineage_hash(
                    self.objective_hash,
                    self.dependency_hash,
                    self.ready_set_hash,
                    self.plan_revision,
                    self.completed_tasks,
                )
            && self
                .tasks
                .iter()
                .copied()
                .all(|task| task.is_valid(self.objective_id))
            && self
                .tasks
                .windows(2)
                .all(|pair| pair[0].ordinal < pair[1].ordinal)
            && self.tasks.iter().filter(|task| task.ready).count() == usize::from(self.ready_tasks)
            && self
                .tasks
                .iter()
                .any(|task| task.ready && task.task_id == self.task_id)
    }

    pub fn submission(&self) -> EvidenceSubmission {
        let passed = self.decision() == PlanDecision::Ready;
        EvidenceSubmission::with_effect_payload(
            GateId::Plan,
            Evidence::TaskReady,
            passed,
            if passed {
                PacketEffect::BindReadyTask
            } else {
                PacketEffect::None
            },
            plan_payload_hash(self),
        )
    }

    pub fn receipt(&self) -> PlanReceipt {
        PlanReceipt::from_record(self)
    }
}

impl PlanReceipt {
    pub fn from_record(record: &PlanRecord) -> Self {
        let mut receipt = Self {
            schema_version: PLAN_RECEIPT_SCHEMA_VERSION,
            record_type: PLAN_RECEIPT_RECORD,
            objective_id: record.objective_id,
            task_id: record.task_id,
            task_count: u64::from(record.task_count),
            completed_tasks: u64::from(record.completed_tasks),
            ready_tasks: u64::from(record.ready_tasks),
            plan_version: record.plan_version,
            plan_revision: record.plan_revision,
            objective_hash: record.objective_hash,
            dependency_hash: record.dependency_hash,
            ready_set_hash: record.ready_set_hash,
            lineage_hash: record.lineage_hash,
            payload_hash: plan_payload_hash(record),
            verdict: record.decision(),
            receipt_hash: 0,
        };
        receipt.receipt_hash = receipt.expected_receipt_hash();
        receipt
    }

    pub fn is_valid_for(self, record: &PlanRecord) -> bool {
        self == PlanReceipt::from_record(record) && self.is_self_consistent()
    }

    pub fn is_self_consistent(self) -> bool {
        self.schema_version == PLAN_RECEIPT_SCHEMA_VERSION
            && self.record_type == PLAN_RECEIPT_RECORD
            && self.objective_id != 0
            && self.task_count != 0
            && self.completed_tasks <= self.task_count
            && self.plan_version == PLAN_SCHEMA_VERSION
            && self.plan_revision != 0
            && self.objective_hash != 0
            && self.dependency_hash != 0
            && self.ready_set_hash != 0
            && self.lineage_hash != 0
            && self.payload_hash != 0
            && self.receipt_hash == self.expected_receipt_hash()
            && match self.verdict {
                PlanDecision::Ready => self.task_id != 0 && self.ready_tasks != 0,
                PlanDecision::Blocked => self.ready_tasks == 0,
            }
    }

    pub fn submission(self) -> EvidenceSubmission {
        let passed = self.is_self_consistent() && self.verdict == PlanDecision::Ready;
        EvidenceSubmission::with_effect_payload(
            GateId::Plan,
            Evidence::TaskReady,
            passed,
            if passed {
                PacketEffect::BindReadyTask
            } else {
                PacketEffect::None
            },
            self.receipt_hash,
        )
    }

    pub fn expected_receipt_hash(self) -> u64 {
        let mut h = 0x91a0_0001_5eed_0001u64;
        h = mix(h, self.schema_version);
        h = mix(h, self.record_type);
        h = mix(h, self.objective_id);
        h = mix(h, self.task_id);
        h = mix(h, self.task_count);
        h = mix(h, self.completed_tasks);
        h = mix(h, self.ready_tasks);
        h = mix(h, self.plan_version);
        h = mix(h, self.plan_revision);
        h = mix(h, self.objective_hash);
        h = mix(h, self.dependency_hash);
        h = mix(h, self.ready_set_hash);
        h = mix(h, self.lineage_hash);
        h = mix(h, self.payload_hash);
        h = mix(h, self.verdict as u64);
        h.max(1)
    }
}

impl EvidenceProducer for PlanRecord {
    type Record = PlanRecord;

    fn record(&self) -> &Self::Record {
        self
    }

    fn submission(&self) -> EvidenceSubmission {
        PlanRecord::submission(self)
    }
}

fn plan_tasks(objective_id: u64, required: u8, completed: u8) -> Vec<PlanTask> {
    (0..required)
        .map(|idx| {
            let ordinal = idx.saturating_add(1);
            let task_id = objective_id
                .saturating_mul(100)
                .saturating_add(u64::from(ordinal));
            let depends_on = if ordinal == 1 {
                0
            } else {
                objective_id
                    .saturating_mul(100)
                    .saturating_add(u64::from(ordinal.saturating_sub(1)))
            };
            PlanTask {
                task_id,
                depends_on,
                ordinal,
                ready: idx == completed,
            }
        })
        .collect()
}

fn objective_hash(packet: Packet) -> u64 {
    let mut h = 0x4f42_4a45_4354_0001u64;
    h = mix(h, packet.objective_id);
    h = mix(h, u64::from(packet.objective_required_tasks));
    h = mix(h, packet.revision.max(1));
    h.max(1)
}

fn dependency_hash(tasks: &[PlanTask]) -> u64 {
    let mut h = 0x4445_5053_504c_414eu64;
    for task in tasks {
        h = mix(h, task.task_id);
        h = mix(h, task.depends_on);
        h = mix(h, u64::from(task.ordinal));
    }
    h.max(1)
}

fn ready_set_hash(tasks: &[PlanTask]) -> u64 {
    let mut h = 0x5245_4144_595f_5345u64;
    for task in tasks.iter().filter(|task| task.ready) {
        h = mix(h, task.hash());
    }
    h.max(1)
}

fn lineage_hash(
    objective_hash: u64,
    dependency_hash: u64,
    ready_set_hash: u64,
    plan_revision: u64,
    completed_tasks: u8,
) -> u64 {
    let mut h = 0x4c49_4e45_4147_4501u64;
    h = mix(h, PLAN_SCHEMA_VERSION);
    h = mix(h, objective_hash);
    h = mix(h, dependency_hash);
    h = mix(h, ready_set_hash);
    h = mix(h, plan_revision);
    h = mix(h, u64::from(completed_tasks));
    h.max(1)
}

fn plan_payload_hash(record: &PlanRecord) -> u64 {
    let mut h = 0x9e37_79b9_7f4a_7c15u64;
    h = mix(h, record.objective_id);
    h = mix(h, record.task_id);
    h = mix(h, u64::from(record.ready_tasks));
    h = mix(h, record.dependency_hash);
    h = mix(h, record.objective_hash);
    h = mix(h, record.ready_set_hash);
    h = mix(h, record.lineage_hash);
    h.max(1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::Packet;

    fn packet(required: u8, completed: u8) -> Packet {
        let mut packet = Packet::empty();
        packet.objective_id = 42;
        packet.objective_required_tasks = required;
        packet.objective_done_tasks = completed;
        packet.revision = 9;
        packet
    }

    #[test]
    fn plan_receipt_binds_lineage_and_payload() {
        let record = PlanRecord::from_packet(packet(4, 1));
        let receipt = record.receipt();

        assert!(receipt.is_self_consistent());
        assert!(receipt.is_valid_for(&record));
        assert_eq!(receipt.verdict, PlanDecision::Ready);
        assert_eq!(receipt.task_id, 4202);
        assert_eq!(receipt.task_count, 4);
        assert_eq!(receipt.completed_tasks, 1);
        assert_eq!(receipt.ready_tasks, 1);
        assert_eq!(receipt.payload_hash, plan_payload_hash(&record));
        assert_eq!(receipt.submission().gate, GateId::Plan);
        assert!(receipt.submission().passed);
    }

    #[test]
    fn plan_receipt_rejects_tampered_lineage() {
        let record = PlanRecord::from_packet(packet(3, 0));
        let mut receipt = record.receipt();
        receipt.lineage_hash ^= 1;

        assert!(!receipt.is_self_consistent());
        assert!(!receipt.is_valid_for(&record));
        assert!(!receipt.submission().passed);
    }

    #[test]
    fn plan_receipt_blocks_completed_plan() {
        let record = PlanRecord::from_packet(packet(2, 2));
        let receipt = record.receipt();

        assert_eq!(receipt.verdict, PlanDecision::Blocked);
        assert!(receipt.is_self_consistent());
        assert!(receipt.is_valid_for(&record));
        assert!(!receipt.submission().passed);
    }
}
