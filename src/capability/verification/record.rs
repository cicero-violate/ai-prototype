//! Durable verification payload owned by the verification capability.
//!
//! Verification is the second capability boundary that must stop being only a
//! passive record constructor. This module models the minimum deterministic
//! verification loop:
//!
//! ```text
//! VerificationRequest -> DeterministicSemanticVerifier -> SemanticVerificationReceipt -> EvidenceSubmission
//! ```
//!
//! The verifier intentionally does not inspect real files yet. It gives the
//! runtime a typed request/receipt seam that can later be backed by filesystem,
//! diff, AST, test, or external semantic validators without changing the kernel
//! evidence contract.

use crate::capability::{EvidenceProducer, EvidenceSubmission, PacketEffect};
use crate::kernel::{mix, Evidence, GateId, Packet};

const DEFECT_NONE: u64 = 0;
const DEFECT_REQUEST_DENIED: u64 = 1 << 0;
const DEFECT_PROFILE_STRUCTURAL: u64 = 1 << 1;
const DEFECT_RECEIPT_MISMATCH: u64 = 1 << 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VerificationDecision {
    Accepted,
    Rejected,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum VerificationCheck {
    ArtifactSemantics = 1,
    Denied = 2,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VerificationRequest {
    pub check: VerificationCheck,
    pub profile: ArtifactSemanticProfile,
    pub requested_effect: PacketEffect,
}

impl VerificationRequest {
    pub fn from_packet(packet: Packet) -> Self {
        Self::from_profile(ArtifactSemanticProfile::from_packet(packet))
    }

    pub fn from_profile(profile: ArtifactSemanticProfile) -> Self {
        Self {
            check: if profile.artifact_id != 0 {
                VerificationCheck::ArtifactSemantics
            } else {
                VerificationCheck::Denied
            },
            profile,
            requested_effect: PacketEffect::RepairLineage,
        }
    }

    pub fn is_admissible(self) -> bool {
        self.check == VerificationCheck::ArtifactSemantics
            && self.requested_effect == PacketEffect::RepairLineage
            && self.profile.is_structurally_valid()
    }

    pub fn contract_hash(self) -> u64 {
        let mut h = 0x13198a2e03707344u64;
        h = mix(h, self.check as u64);
        h = mix(h, self.profile.semantic_hash());
        h = mix(h, self.profile.expected_receipt_hash());
        h = mix(h, self.profile.expected_lineage_hash());
        h = mix(h, self.requested_effect as u64);
        h.max(1)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SemanticVerificationReceipt {
    pub request_hash: u64,
    pub artifact_id: u64,
    pub receipt_hash: u64,
    pub expected_receipt_hash: u64,
    pub expected_lineage_hash: u64,
    pub semantic_profile_hash: u64,
    pub semantic_check_hash: u64,
    pub defect_mask: u64,
}

impl SemanticVerificationReceipt {
    pub fn is_accepted_for(self, request: VerificationRequest) -> bool {
        self.defect_mask == DEFECT_NONE
            && self.request_hash == request.contract_hash()
            && self.artifact_id == request.profile.artifact_id
            && self.receipt_hash == request.profile.receipt_hash
            && self.expected_receipt_hash == request.profile.expected_receipt_hash()
            && self.expected_lineage_hash == request.profile.expected_lineage_hash()
            && self.semantic_profile_hash == request.profile.semantic_hash()
            && self.semantic_check_hash == verification_hash(request.profile)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct DeterministicSemanticVerifier;

impl DeterministicSemanticVerifier {
    pub fn verify(self, request: VerificationRequest) -> SemanticVerificationReceipt {
        let mut defect_mask = DEFECT_NONE;

        if request.check != VerificationCheck::ArtifactSemantics
            || request.requested_effect != PacketEffect::RepairLineage
        {
            defect_mask |= DEFECT_REQUEST_DENIED;
        }

        if !request.profile.is_structurally_valid() {
            defect_mask |= DEFECT_PROFILE_STRUCTURAL;
        }

        if !request.profile.receipt_valid() {
            defect_mask |= DEFECT_RECEIPT_MISMATCH;
        }

        SemanticVerificationReceipt {
            request_hash: request.contract_hash(),
            artifact_id: request.profile.artifact_id,
            receipt_hash: request.profile.receipt_hash,
            expected_receipt_hash: request.profile.expected_receipt_hash(),
            expected_lineage_hash: request.profile.expected_lineage_hash(),
            semantic_profile_hash: request.profile.semantic_hash(),
            semantic_check_hash: verification_hash(request.profile),
            defect_mask,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerificationRecord {
    pub request: VerificationRequest,
    pub receipt: SemanticVerificationReceipt,
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
        Self::from_request(VerificationRequest::from_packet(packet))
    }

    pub fn from_profile(profile: ArtifactSemanticProfile) -> Self {
        Self::from_request(VerificationRequest::from_profile(profile))
    }

    pub fn from_request(request: VerificationRequest) -> Self {
        let receipt = DeterministicSemanticVerifier::default().verify(request);
        Self {
            request,
            receipt,
            artifact_id: request.profile.artifact_id,
            receipt_hash: request.profile.receipt_hash,
            expected_receipt_hash: receipt.expected_receipt_hash,
            expected_lineage_hash: receipt.expected_lineage_hash,
            semantic_profile: request.profile,
            semantic_profile_hash: receipt.semantic_profile_hash,
            semantic_check_hash: receipt.semantic_check_hash,
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
        self.request.is_admissible()
            && self.receipt.is_accepted_for(self.request)
            && self.artifact_id != 0
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
        EvidenceSubmission::with_effect_payload(
            GateId::Verification,
            Evidence::LineageProof,
            passed,
            if passed {
                PacketEffect::RepairLineage
            } else {
                PacketEffect::None
            },
            verification_payload_hash(self),
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

fn verification_payload_hash(record: &VerificationRecord) -> u64 {
    let mut h = 0xa54ff53a5f1d36f1u64;
    h = mix(h, record.request.contract_hash());
    h = mix(h, record.receipt.request_hash);
    h = mix(h, record.receipt.artifact_id);
    h = mix(h, record.receipt.receipt_hash);
    h = mix(h, record.receipt.expected_receipt_hash);
    h = mix(h, record.receipt.expected_lineage_hash);
    h = mix(h, record.receipt.semantic_profile_hash);
    h = mix(h, record.receipt.semantic_check_hash);
    h = mix(h, record.receipt.defect_mask);
    h.max(1)
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