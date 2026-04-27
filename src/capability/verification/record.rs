//! Durable verification payload owned by the verification capability.

use crate::capability::{EvidenceProducer, EvidenceSubmission, PacketEffect};
use crate::kernel::{mix, Evidence, GateId, Packet};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VerificationDecision {
    Accepted,
    Rejected,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ArtifactSemanticProfile {
    pub objective_id: u64,
    pub objective_required_tasks: u8,
    pub objective_done_tasks: u8,
    pub ready_tasks: u8,
    pub active_task_id: u64,
    pub artifact_id: u64,
    pub parent_artifact_id: u64,
    pub artifact_bytes: u64,
    pub receipt_hash: u64,
    pub lineage_hash: u64,
    pub revision: u64,
}

impl ArtifactSemanticProfile {
    pub fn from_packet(packet: Packet) -> Self {
        Self {
            objective_id: packet.objective_id,
            objective_required_tasks: packet.objective_required_tasks,
            objective_done_tasks: packet.objective_done_tasks,
            ready_tasks: packet.ready_tasks,
            active_task_id: packet.active_task_id,
            artifact_id: packet.artifact_id,
            parent_artifact_id: packet.parent_artifact_id,
            artifact_bytes: packet.artifact_bytes,
            receipt_hash: packet.artifact_receipt_hash,
            lineage_hash: packet.artifact_lineage_hash,
            revision: packet.revision,
        }
    }

    pub fn is_structurally_valid(self) -> bool {
        self.objective_id != 0
            && self.objective_required_tasks != 0
            && self.objective_done_tasks <= self.objective_required_tasks
            && self.active_task_id != 0
            && self.artifact_id != 0
            && self.artifact_bytes != 0
            && self.receipt_hash != 0
            && self.expected_receipt_hash() != 0
            && self.expected_lineage_hash() != 0
            && self.semantic_hash() != 0
    }

    pub fn receipt_valid(self) -> bool {
        self.receipt_hash == self.expected_receipt_hash()
    }

    pub fn lineage_valid(self) -> bool {
        self.receipt_valid() && self.lineage_hash == self.expected_lineage_hash()
    }

    pub fn expected_receipt_hash(self) -> u64 {
        let mut h = 0x243f6a8885a308d3u64;
        h = mix(h, self.objective_id);
        h = mix(h, self.active_task_id);
        h = mix(h, self.parent_artifact_id);
        h = mix(h, self.artifact_id);
        h = mix(h, self.artifact_bytes);
        h = mix(h, self.revision);
        h
    }

    pub fn expected_lineage_hash(self) -> u64 {
        let mut h = 0x9e3779b97f4a7c15u64;
        h = mix(h, self.objective_id);
        h = mix(h, self.active_task_id);
        h = mix(h, self.parent_artifact_id);
        h = mix(h, self.artifact_id);
        h = mix(h, self.artifact_bytes);
        h = mix(h, self.receipt_hash);
        h = mix(h, self.revision);
        h
    }

    pub fn semantic_hash(self) -> u64 {
        let mut h = 0x7f4a7c159e3779b9u64;
        h = mix(h, self.objective_id);
        h = mix(h, self.objective_required_tasks as u64);
        h = mix(h, self.objective_done_tasks as u64);
        h = mix(h, self.ready_tasks as u64);
        h = mix(h, self.active_task_id);
        h = mix(h, self.artifact_id);
        h = mix(h, self.parent_artifact_id);
        h = mix(h, self.artifact_bytes);
        h = mix(h, self.receipt_hash);
        h = mix(h, self.lineage_hash);
        h = mix(h, self.revision);
        h.max(1)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerificationRecord {
    pub artifact_id: u64,
    pub receipt_hash: u64,
    pub expected_receipt_hash: u64,
    pub expected_lineage_hash: u64,
    pub semantic_profile: ArtifactSemanticProfile,
    pub semantic_profile_hash: u64,
    pub semantic_check_hash: u64,
}

impl VerificationRecord {
    pub fn from_packet(packet: Packet) -> Self {
        Self::from_profile(ArtifactSemanticProfile::from_packet(packet))
    }

    pub fn from_profile(profile: ArtifactSemanticProfile) -> Self {
        Self {
            artifact_id: profile.artifact_id,
            receipt_hash: profile.receipt_hash,
            expected_receipt_hash: profile.expected_receipt_hash(),
            expected_lineage_hash: profile.expected_lineage_hash(),
            semantic_profile: profile,
            semantic_profile_hash: profile.semantic_hash(),
            semantic_check_hash: verification_hash(profile),
        }
    }

    pub fn decision(&self) -> VerificationDecision {
        if self.is_valid() {
            VerificationDecision::Accepted
        } else {
            VerificationDecision::Rejected
        }
    }

    pub fn is_valid(&self) -> bool {
        self.artifact_id != 0
            && self.receipt_hash != 0
            && self.expected_receipt_hash != 0
            && self.expected_lineage_hash != 0
            && self.semantic_profile_hash != 0
            && self.semantic_check_hash != 0
            && self.semantic_profile.is_structurally_valid()
            && self.artifact_id == self.semantic_profile.artifact_id
            && self.receipt_hash == self.semantic_profile.receipt_hash
            && self.expected_receipt_hash == self.semantic_profile.expected_receipt_hash()
            && self.receipt_hash == self.expected_receipt_hash
            && self.expected_lineage_hash == self.semantic_profile.expected_lineage_hash()
            && self.semantic_profile_hash == self.semantic_profile.semantic_hash()
            && self.semantic_check_hash == verification_hash(self.semantic_profile)
    }

    pub fn lineage_already_valid(&self) -> bool {
        self.is_valid() && self.semantic_profile.lineage_valid()
    }

    pub fn submission(&self) -> EvidenceSubmission {
        let passed = self.decision() == VerificationDecision::Accepted;
        EvidenceSubmission::with_effect(
            GateId::Verification,
            Evidence::LineageProof,
            passed,
            if passed {
                PacketEffect::RepairLineage
            } else {
                PacketEffect::None
            },
        )
    }
}

impl EvidenceProducer for VerificationRecord {
    type Record = VerificationRecord;

    fn record(&self) -> &Self::Record {
        self
    }

    fn submission(&self) -> EvidenceSubmission {
        VerificationRecord::submission(self)
    }
}

fn verification_hash(profile: ArtifactSemanticProfile) -> u64 {
    let mut h = 0xa54ff53a5f1d36f1u64;
    h = mix(h, profile.artifact_id);
    h = mix(h, profile.receipt_hash);
    h = mix(h, profile.expected_receipt_hash());
    h = mix(h, profile.expected_lineage_hash());
    h = mix(h, profile.semantic_hash());
    h.max(1)
}