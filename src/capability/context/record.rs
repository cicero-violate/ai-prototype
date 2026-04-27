//! Durable context payload owned by the context capability.

use crate::capability::memory::MemoryLookupRecord;
use crate::capability::{EvidenceProducer, EvidenceSubmission};
use crate::kernel::{Evidence, GateId, Packet};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContextDecision {
    Assembled,
    Insufficient,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContextRecord {
    pub objective_id: u64,
    pub observation_hash: u64,
    pub memory_aggregate_hash: u64,
    pub prior_count: u8,
    pub context_hash: u64,
}

impl ContextRecord {
    pub fn from_packet_memory(
        packet: Packet,
        observation_hash: u64,
        memory: &MemoryLookupRecord,
    ) -> Self {
        let prior_count = memory.match_count();
        let memory_aggregate_hash = memory.aggregate_hash;
        Self {
            objective_id: packet.objective_id,
            observation_hash,
            memory_aggregate_hash,
            prior_count,
            context_hash: context_hash(packet, observation_hash, memory_aggregate_hash, prior_count),
        }
    }

    pub fn decision(&self) -> ContextDecision {
        if self.is_valid() {
            ContextDecision::Assembled
        } else {
            ContextDecision::Insufficient
        }
    }

    pub fn is_valid(&self) -> bool {
        self.objective_id != 0
            && self.observation_hash != 0
            && self.memory_aggregate_hash != 0
            && self.context_hash != 0
    }

    pub fn submission(&self) -> EvidenceSubmission {
        EvidenceSubmission::new(
            GateId::Analysis,
            Evidence::AnalysisReport,
            self.decision() == ContextDecision::Assembled,
        )
    }
}

impl EvidenceProducer for ContextRecord {
    type Record = ContextRecord;

    fn record(&self) -> &Self::Record {
        self
    }

    fn submission(&self) -> EvidenceSubmission {
        ContextRecord::submission(self)
    }
}

fn context_hash(
    packet: Packet,
    observation_hash: u64,
    memory_aggregate_hash: u64,
    prior_count: u8,
) -> u64 {
    let mut h = 0x1f83d9abfb41bd6bu64;
    h = mix(h, packet.objective_id);
    h = mix(h, packet.objective_required_tasks as u64);
    h = mix(h, packet.revision);
    h = mix(h, observation_hash);
    h = mix(h, memory_aggregate_hash);
    h = mix(h, prior_count as u64);
    h.max(1)
}

fn mix(mut h: u64, x: u64) -> u64 {
    h ^= x;
    h = h.wrapping_mul(0x100000001b3);
    h
}