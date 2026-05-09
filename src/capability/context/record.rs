//! Durable context payload owned by the context capability.

use crate::capability::memory::{MemoryLookupReceipt, MemoryLookupRecord};
use crate::capability::{EvidenceProducer, EvidenceSubmission};
use crate::kernel::{Evidence, GateId, Packet, mix};

pub const CONTEXT_ASSEMBLY_RECEIPT_SCHEMA_VERSION: u64 = 1;
pub const CONTEXT_ASSEMBLY_RECEIPT_RECORD: u64 = 0xc07e_1001;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContextDecision {
    Assembled,
    Insufficient,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContextRecord {
    pub objective_id: u64,
    pub objective_required_tasks: u8,
    pub revision: u64,
    pub observation_hash: u64,
    pub memory_aggregate_hash: u64,
    pub memory_receipt_hash: u64,
    pub prior_count: u8,
    pub context_hash: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ContextAssemblyReceipt {
    pub schema_version: u64,
    pub record_type: u64,
    pub objective_id: u64,
    pub objective_required_tasks: u64,
    pub revision: u64,
    pub observation_hash: u64,
    pub memory_aggregate_hash: u64,
    pub memory_receipt_hash: u64,
    pub prior_count: u64,
    pub context_hash: u64,
    pub verdict: ContextDecision,
    pub receipt_hash: u64,
}

impl ContextRecord {
    pub fn from_packet_memory(
        packet: Packet,
        observation_hash: u64,
        memory: &MemoryLookupRecord,
    ) -> Self {
        let prior_count = memory.match_count();
        let memory_aggregate_hash = memory.aggregate_hash;
        let memory_receipt_hash = legacy_memory_receipt_hash(memory);
        Self {
            objective_id: packet.objective_id,
            objective_required_tasks: packet.objective_required_tasks,
            revision: packet.revision,
            observation_hash,
            memory_aggregate_hash,
            memory_receipt_hash,
            prior_count,
            context_hash: context_hash(
                packet,
                observation_hash,
                memory_aggregate_hash,
                memory_receipt_hash,
                prior_count,
            ),
        }
    }

    pub fn from_packet_memory_receipt(
        packet: Packet,
        observation_hash: u64,
        memory: &MemoryLookupRecord,
        memory_receipt: Option<&MemoryLookupReceipt>,
    ) -> Self {
        let prior_count = memory.match_count();
        let memory_aggregate_hash = memory.aggregate_hash;
        let memory_receipt_hash = memory_receipt
            .filter(|receipt| receipt.is_valid_for(memory))
            .map(|receipt| receipt.receipt_hash)
            .unwrap_or(0);
        Self {
            objective_id: packet.objective_id,
            objective_required_tasks: packet.objective_required_tasks,
            revision: packet.revision,
            observation_hash,
            memory_aggregate_hash,
            memory_receipt_hash,
            prior_count,
            context_hash: context_hash(
                packet,
                observation_hash,
                memory_aggregate_hash,
                memory_receipt_hash,
                prior_count,
            ),
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
            && self.objective_required_tasks != 0
            && self.observation_hash != 0
            && self.memory_aggregate_hash != 0
            && self.memory_receipt_hash != 0
            && self.context_hash != 0
            && self.context_hash
                == context_hash(
                    Packet {
                        objective_id: self.objective_id,
                        objective_required_tasks: self.objective_required_tasks,
                        revision: self.revision,
                        ..Packet::empty()
                    },
                    self.observation_hash,
                    self.memory_aggregate_hash,
                    self.memory_receipt_hash,
                    self.prior_count,
                )
    }

    pub fn submission(&self) -> EvidenceSubmission {
        EvidenceSubmission::with_payload(
            GateId::Analysis,
            Evidence::AnalysisReport,
            self.decision() == ContextDecision::Assembled,
            self.context_hash,
        )
    }

    pub fn receipt(&self) -> ContextAssemblyReceipt {
        ContextAssemblyReceipt::from_record(self)
    }
}

impl ContextAssemblyReceipt {
    pub fn from_record(record: &ContextRecord) -> Self {
        let mut receipt = Self {
            schema_version: CONTEXT_ASSEMBLY_RECEIPT_SCHEMA_VERSION,
            record_type: CONTEXT_ASSEMBLY_RECEIPT_RECORD,
            objective_id: record.objective_id,
            objective_required_tasks: u64::from(record.objective_required_tasks),
            revision: record.revision,
            observation_hash: record.observation_hash,
            memory_aggregate_hash: record.memory_aggregate_hash,
            memory_receipt_hash: record.memory_receipt_hash,
            prior_count: u64::from(record.prior_count),
            context_hash: record.context_hash,
            verdict: record.decision(),
            receipt_hash: 0,
        };
        receipt.receipt_hash = receipt.expected_receipt_hash();
        receipt
    }

    pub fn is_valid_for(self, record: &ContextRecord) -> bool {
        self == ContextAssemblyReceipt::from_record(record) && self.is_self_consistent()
    }

    pub fn is_self_consistent(self) -> bool {
        self.schema_version == CONTEXT_ASSEMBLY_RECEIPT_SCHEMA_VERSION
            && self.record_type == CONTEXT_ASSEMBLY_RECEIPT_RECORD
            && self.objective_id != 0
            && self.objective_required_tasks != 0
            && self.observation_hash != 0
            && self.memory_aggregate_hash != 0
            && self.context_hash != 0
            && self.receipt_hash == self.expected_receipt_hash()
            && match self.verdict {
                ContextDecision::Assembled => self.memory_receipt_hash != 0,
                ContextDecision::Insufficient => true,
            }
    }

    pub fn submission(self) -> EvidenceSubmission {
        EvidenceSubmission::with_payload(
            GateId::Analysis,
            Evidence::AnalysisReport,
            self.is_self_consistent() && self.verdict == ContextDecision::Assembled,
            self.receipt_hash,
        )
    }

    pub fn expected_receipt_hash(self) -> u64 {
        let mut h = 0xc07e_1001_a55e_0001u64;
        h = mix(h, self.schema_version);
        h = mix(h, self.record_type);
        h = mix(h, self.objective_id);
        h = mix(h, self.objective_required_tasks);
        h = mix(h, self.revision);
        h = mix(h, self.observation_hash);
        h = mix(h, self.memory_aggregate_hash);
        h = mix(h, self.memory_receipt_hash);
        h = mix(h, self.prior_count);
        h = mix(h, self.context_hash);
        h = mix(h, self.verdict as u64);
        h.max(1)
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

fn legacy_memory_receipt_hash(memory: &MemoryLookupRecord) -> u64 {
    if !memory.is_valid() {
        return 0;
    }
    let mut h = 0x434f_4e54_4558_4c45u64;
    h = mix(h, memory.query_hash);
    h = mix(h, memory.aggregate_hash);
    h = mix(h, memory.match_count() as u64);
    h.max(1)
}

fn context_hash(
    packet: Packet,
    observation_hash: u64,
    memory_aggregate_hash: u64,
    memory_receipt_hash: u64,
    prior_count: u8,
) -> u64 {
    let mut h = 0x1f83d9abfb41bd6bu64;
    h = mix(h, packet.objective_id);
    h = mix(h, packet.objective_required_tasks as u64);
    h = mix(h, packet.revision);
    h = mix(h, observation_hash);
    h = mix(h, memory_aggregate_hash);
    h = mix(h, memory_receipt_hash);
    h = mix(h, prior_count as u64);
    h.max(1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::memory::{MemoryFact, MemoryIndex};

    fn packet() -> Packet {
        Packet {
            objective_id: 42,
            objective_required_tasks: 3,
            revision: 7,
            ..Packet::empty()
        }
    }

    #[test]
    fn context_record_binds_valid_memory_lookup_receipt() {
        let mut index = MemoryIndex::default();
        assert!(index.insert(MemoryFact::new(42, 700, 9, 1)));
        let (memory, receipt) = index.lookup_with_receipt(42, 4);

        let context =
            ContextRecord::from_packet_memory_receipt(packet(), 900, &memory, Some(&receipt));

        assert_eq!(context.memory_receipt_hash, receipt.receipt_hash);
        assert_eq!(context.decision(), ContextDecision::Assembled);
        assert!(context.submission().passed);
    }

    #[test]
    fn legacy_context_constructor_remains_deterministic_without_lookup_receipt() {
        let mut index = MemoryIndex::default();
        assert!(index.insert(MemoryFact::new(42, 700, 9, 1)));
        let (memory, receipt) = index.lookup_with_receipt(42, 4);

        let legacy = ContextRecord::from_packet_memory(packet(), 900, &memory);
        let receipt_bound =
            ContextRecord::from_packet_memory_receipt(packet(), 900, &memory, Some(&receipt));

        assert_ne!(legacy.memory_receipt_hash, 0);
        assert_ne!(
            legacy.memory_receipt_hash,
            receipt_bound.memory_receipt_hash
        );
        assert_eq!(legacy.decision(), ContextDecision::Assembled);
        assert!(legacy.submission().passed);
    }

    #[test]
    fn context_record_rejects_tampered_memory_receipt() {
        let mut index = MemoryIndex::default();
        assert!(index.insert(MemoryFact::new(42, 700, 9, 1)));
        let (mut memory, receipt) = index.lookup_with_receipt(42, 4);
        memory.matches.clear();

        let context =
            ContextRecord::from_packet_memory_receipt(packet(), 900, &memory, Some(&receipt));

        assert_eq!(context.memory_receipt_hash, 0);
        assert_eq!(context.decision(), ContextDecision::Insufficient);
    }

    #[test]
    fn context_record_rejects_hash_tampering() {
        let mut index = MemoryIndex::default();
        assert!(index.insert(MemoryFact::new(42, 700, 9, 1)));
        let (memory, receipt) = index.lookup_with_receipt(42, 4);
        let mut context =
            ContextRecord::from_packet_memory_receipt(packet(), 900, &memory, Some(&receipt));
        context.memory_receipt_hash ^= 1;

        assert!(!context.is_valid());
    }

    #[test]
    fn context_assembly_receipt_binds_analysis_inputs() {
        let mut index = MemoryIndex::default();
        assert!(index.insert(MemoryFact::new(42, 700, 9, 1)));
        let (memory, lookup_receipt) = index.lookup_with_receipt(42, 4);
        let context = ContextRecord::from_packet_memory_receipt(
            packet(),
            900,
            &memory,
            Some(&lookup_receipt),
        );
        let receipt = context.receipt();

        assert!(receipt.is_self_consistent());
        assert!(receipt.is_valid_for(&context));
        assert_eq!(receipt.verdict, ContextDecision::Assembled);
        assert_eq!(receipt.memory_receipt_hash, lookup_receipt.receipt_hash);
        assert_eq!(receipt.context_hash, context.context_hash);
        assert_eq!(receipt.submission().gate, GateId::Analysis);
        assert!(receipt.submission().passed);
    }

    #[test]
    fn context_assembly_receipt_rejects_tampered_context_hash() {
        let mut index = MemoryIndex::default();
        assert!(index.insert(MemoryFact::new(42, 700, 9, 1)));
        let (memory, lookup_receipt) = index.lookup_with_receipt(42, 4);
        let context = ContextRecord::from_packet_memory_receipt(
            packet(),
            900,
            &memory,
            Some(&lookup_receipt),
        );
        let mut receipt = context.receipt();
        receipt.context_hash ^= 1;

        assert!(!receipt.is_self_consistent());
        assert!(!receipt.is_valid_for(&context));
        assert!(!receipt.submission().passed);
    }

    #[test]
    fn context_assembly_receipt_records_insufficient_context_without_gate_pass() {
        let context = ContextRecord {
            objective_id: 42,
            objective_required_tasks: 3,
            revision: 7,
            observation_hash: 900,
            memory_aggregate_hash: 700,
            memory_receipt_hash: 0,
            prior_count: 1,
            context_hash: context_hash(packet(), 900, 700, 0, 1),
        };
        let receipt = context.receipt();

        assert_eq!(receipt.verdict, ContextDecision::Insufficient);
        assert!(receipt.is_self_consistent());
        assert!(receipt.is_valid_for(&context));
        assert!(!receipt.submission().passed);
    }
}
