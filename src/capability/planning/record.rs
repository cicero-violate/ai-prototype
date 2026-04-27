//! Durable planning payload owned by the planning capability.

use crate::capability::{EvidenceProducer, EvidenceSubmission, PacketEffect};
use crate::kernel::{mix, Evidence, GateId, Packet};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlanDecision {
    Ready,
    Blocked,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlanRecord {
    pub objective_id: u64,
    pub task_id: u64,
    pub ready_tasks: u8,
    pub dependency_hash: u64,
}

impl PlanRecord {
    pub fn from_packet(packet: Packet) -> Self {
        Self {
            objective_id: packet.objective_id,
            task_id: packet.objective_id.saturating_mul(100).saturating_add(1),
            ready_tasks: 1,
            dependency_hash: plan_dependency_hash(packet),
        }
    }

    pub fn decision(&self) -> PlanDecision {
        if self.is_valid() {
            PlanDecision::Ready
        } else {
            PlanDecision::Blocked
        }
    }

    pub fn is_valid(&self) -> bool {
        self.objective_id != 0
            && self.task_id != 0
            && self.ready_tasks != 0
            && self.dependency_hash != 0
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

fn plan_payload_hash(record: &PlanRecord) -> u64 {
    let mut h = 0x9e37_79b9_7f4a_7c15u64;
    h = mix(h, record.objective_id);
    h = mix(h, record.task_id);
    h = mix(h, record.ready_tasks as u64);
    h = mix(h, record.dependency_hash);
    h.max(1)
}

fn plan_dependency_hash(packet: Packet) -> u64 {
    let mut h = 0x6a09e667f3bcc909u64;
    h ^= packet.objective_id;
    h = h.wrapping_mul(0x100000001b3);
    h ^= packet.objective_required_tasks as u64;
    h = h.wrapping_mul(0x100000001b3);
    h ^= packet.revision;
    h = h.wrapping_mul(0x100000001b3);
    h.max(1)
}