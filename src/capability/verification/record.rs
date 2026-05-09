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
//! The verifier keeps packet semantics deterministic and also exposes
//! artifact-backed profiles that inspect concrete files without changing the
//! kernel evidence contract.

use crate::capability::{EvidenceProducer, EvidenceSubmission, PacketEffect};
use crate::kernel::{mix, Evidence, GateId, Packet};
use std::fs;
use std::path::Path;

pub use super::proof::{
    append_verification_proof_record_ndjson, decode_verification_proof_record_ndjson,
    encode_verification_proof_record_ndjson, load_verification_proof_records_ndjson,
    verify_verification_proof_record_bindings, verify_verification_proof_record_order_ndjson,
    verify_verification_proof_record_replay, verify_verification_proof_record_replay_ndjson,
    verify_verification_proof_records, verify_verification_proof_records_ndjson, ProofSubjectKind,
    VerificationProofBinding, VerificationProofError, VerificationProofRecord,
    PROOF_FLAGS_REQUIRED, PROOF_FLAG_PHASE_VERIFIED, PROOF_FLAG_PROVENANCE_VERIFIED,
    PROOF_FLAG_RECEIPT_VERIFIED, PROOF_FLAG_TAMPER_REJECTED, VERIFICATION_PROOF_RECORD,
    VERIFICATION_PROOF_SCHEMA_VERSION,
};

const DEFECT_NONE: u64 = 0;
const DEFECT_REQUEST_DENIED: u64 = 1 << 0;
const DEFECT_PROFILE_STRUCTURAL: u64 = 1 << 1;
const DEFECT_RECEIPT_MISMATCH: u64 = 1 << 2;
const DEFECT_ARTIFACT_MISSING: u64 = 1 << 3;
const DEFECT_ARTIFACT_EMPTY: u64 = 1 << 4;
const DEFECT_ARTIFACT_HASH_MISMATCH: u64 = 1 << 5;
const DEFECT_ARTIFACT_SEMANTICALLY_EMPTY: u64 = 1 << 6;

pub const VERIFICATION_RECEIPT_SCHEMA_VERSION: u64 = 1;
pub const VERIFICATION_RECEIPT_RECORD: u64 = 0x98a0_0001;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ArtifactVerificationProfileKind {
    SourcePatch = 1,
    CommandReceipt = 2,
    TLogNdjson = 3,
    PolicyRow = 4,
    VerificationProofRow = 5,
    DistillationRow = 6,
}

impl ArtifactVerificationProfileKind {
    pub const fn required_token(self) -> &'static str {
        match self {
            Self::SourcePatch => "diff",
            Self::CommandReceipt => "exit_code",
            Self::TLogNdjson => "TLOG",
            Self::PolicyRow => "policy",
            Self::VerificationProofRow => "VERIFICATION_PROOF",
            Self::DistillationRow => "distill",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ArtifactBackedSemanticProfile {
    pub kind: ArtifactVerificationProfileKind,
    pub source_event: u64,
    pub semantic_input_hash: u64,
    pub observable_output_hash: u64,
    pub expected_content_hash: u64,
    pub expected_bytes: u64,
    pub proof_hash: u64,
}

impl ArtifactBackedSemanticProfile {
    pub fn new(
        kind: ArtifactVerificationProfileKind,
        source_event: u64,
        semantic_input_hash: u64,
        observable_output_hash: u64,
        expected_content_hash: u64,
        expected_bytes: u64,
    ) -> Self {
        let proof_hash = artifact_profile_proof_hash(
            kind,
            source_event,
            semantic_input_hash,
            observable_output_hash,
            expected_content_hash,
            expected_bytes,
        );
        Self {
            kind,
            source_event,
            semantic_input_hash,
            observable_output_hash,
            expected_content_hash,
            expected_bytes,
            proof_hash,
        }
    }

    pub fn is_structurally_valid(self) -> bool {
        self.source_event != 0
            && self.semantic_input_hash != 0
            && self.observable_output_hash != 0
            && self.expected_content_hash != 0
            && self.expected_bytes != 0
            && self.proof_hash
                == artifact_profile_proof_hash(
                    self.kind,
                    self.source_event,
                    self.semantic_input_hash,
                    self.observable_output_hash,
                    self.expected_content_hash,
                    self.expected_bytes,
                )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArtifactBackedSemanticReceipt {
    pub profile: ArtifactBackedSemanticProfile,
    pub observed_content_hash: u64,
    pub observed_bytes: u64,
    pub proof_hash: u64,
    pub defect_mask: u64,
}

impl ArtifactBackedSemanticReceipt {
    pub fn is_valid(&self) -> bool {
        self.defect_mask == DEFECT_NONE
            && self.profile.is_structurally_valid()
            && self.observed_content_hash == self.profile.expected_content_hash
            && self.observed_bytes == self.profile.expected_bytes
            && self.proof_hash == self.profile.proof_hash
    }
}

pub fn content_hash(bytes: &[u8]) -> u64 {
    let mut h = 0xcbf29ce484222325u64;
    for byte in bytes {
        h = mix(h, *byte as u64);
    }
    h.max(1)
}

pub fn verify_artifact_backed_semantics(
    path: impl AsRef<Path>,
    profile: ArtifactBackedSemanticProfile,
) -> ArtifactBackedSemanticReceipt {
    let mut defect_mask = DEFECT_NONE;

    if !profile.is_structurally_valid() {
        defect_mask |= DEFECT_PROFILE_STRUCTURAL;
    }

    let bytes = match fs::read(path.as_ref()) {
        Ok(bytes) => bytes,
        Err(_) => {
            return ArtifactBackedSemanticReceipt {
                profile,
                observed_content_hash: 0,
                observed_bytes: 0,
                proof_hash: profile.proof_hash,
                defect_mask: defect_mask | DEFECT_ARTIFACT_MISSING,
            };
        }
    };

    let observed_bytes = bytes.len() as u64;
    let observed_content_hash = content_hash(&bytes);

    if observed_bytes == 0 {
        defect_mask |= DEFECT_ARTIFACT_EMPTY;
    }

    if observed_bytes != profile.expected_bytes
        || observed_content_hash != profile.expected_content_hash
    {
        defect_mask |= DEFECT_ARTIFACT_HASH_MISMATCH;
    }

    let text = String::from_utf8_lossy(&bytes);
    if !text.contains(profile.kind.required_token()) {
        defect_mask |= DEFECT_ARTIFACT_SEMANTICALLY_EMPTY;
    }

    ArtifactBackedSemanticReceipt {
        profile,
        observed_content_hash,
        observed_bytes,
        proof_hash: profile.proof_hash,
        defect_mask,
    }
}

fn artifact_profile_proof_hash(
    kind: ArtifactVerificationProfileKind,
    source_event: u64,
    semantic_input_hash: u64,
    observable_output_hash: u64,
    expected_content_hash: u64,
    expected_bytes: u64,
) -> u64 {
    let mut h = 0x6a09e667f3bcc909u64;
    h = mix(h, kind as u64);
    h = mix(h, source_event);
    h = mix(h, semantic_input_hash);
    h = mix(h, observable_output_hash);
    h = mix(h, expected_content_hash);
    h = mix(h, expected_bytes);
    h.max(1)
}

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VerificationReceipt {
    pub schema_version: u64,
    pub record_type: u64,
    pub request_hash: u64,
    pub artifact_id: u64,
    pub receipt_hash: u64,
    pub expected_receipt_hash: u64,
    pub expected_lineage_hash: u64,
    pub semantic_profile_hash: u64,
    pub semantic_check_hash: u64,
    pub defect_mask: u64,
    pub payload_hash: u64,
    pub verdict: VerificationDecision,
    pub audit_hash: u64,
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

    pub fn receipt(&self) -> VerificationReceipt {
        VerificationReceipt::from_record(self)
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

impl VerificationReceipt {
    pub fn from_record(record: &VerificationRecord) -> Self {
        let mut receipt = Self {
            schema_version: VERIFICATION_RECEIPT_SCHEMA_VERSION,
            record_type: VERIFICATION_RECEIPT_RECORD,
            request_hash: record.request.contract_hash(),
            artifact_id: record.artifact_id,
            receipt_hash: record.receipt_hash,
            expected_receipt_hash: record.expected_receipt_hash,
            expected_lineage_hash: record.expected_lineage_hash,
            semantic_profile_hash: record.semantic_profile_hash,
            semantic_check_hash: record.semantic_check_hash,
            defect_mask: record.receipt.defect_mask,
            payload_hash: verification_payload_hash(record),
            verdict: record.decision(),
            audit_hash: 0,
        };
        receipt.audit_hash = receipt.expected_audit_hash();
        receipt
    }

    pub fn is_valid_for(self, record: &VerificationRecord) -> bool {
        self == VerificationReceipt::from_record(record) && self.is_self_consistent()
    }

    pub fn is_self_consistent(self) -> bool {
        self.schema_version == VERIFICATION_RECEIPT_SCHEMA_VERSION
            && self.record_type == VERIFICATION_RECEIPT_RECORD
            && self.request_hash != 0
            && self.artifact_id != 0
            && self.receipt_hash != 0
            && self.expected_receipt_hash != 0
            && self.expected_lineage_hash != 0
            && self.semantic_profile_hash != 0
            && self.semantic_check_hash != 0
            && self.payload_hash != 0
            && self.audit_hash == self.expected_audit_hash()
            && match self.verdict {
                VerificationDecision::Accepted => self.defect_mask == DEFECT_NONE,
                VerificationDecision::Rejected => self.defect_mask != DEFECT_NONE,
            }
    }

    pub fn submission(self) -> EvidenceSubmission {
        let passed = self.is_self_consistent() && self.verdict == VerificationDecision::Accepted;
        EvidenceSubmission::with_effect_payload(
            GateId::Verification,
            Evidence::LineageProof,
            passed,
            if passed {
                PacketEffect::RepairLineage
            } else {
                PacketEffect::None
            },
            self.audit_hash,
        )
    }

    pub fn expected_audit_hash(self) -> u64 {
        let mut h = 0x98a0_0001_5eed_0001u64;
        h = mix(h, self.schema_version);
        h = mix(h, self.record_type);
        h = mix(h, self.request_hash);
        h = mix(h, self.artifact_id);
        h = mix(h, self.receipt_hash);
        h = mix(h, self.expected_receipt_hash);
        h = mix(h, self.expected_lineage_hash);
        h = mix(h, self.semantic_profile_hash);
        h = mix(h, self.semantic_check_hash);
        h = mix(h, self.defect_mask);
        h = mix(h, self.payload_hash);
        h = mix(h, self.verdict as u64);
        h.max(1)
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

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_packet() -> Packet {
        let mut packet = Packet::empty();
        packet.objective_id = 42;
        packet.objective_required_tasks = 2;
        packet.objective_done_tasks = 1;
        packet.ready_tasks = 1;
        packet.active_task_id = 4202;
        packet.artifact_id = 77;
        packet.parent_artifact_id = 76;
        packet.artifact_bytes = 128;
        packet.revision = 5;

        let profile = ArtifactSemanticProfile::from_packet(packet);
        packet.artifact_receipt_hash = profile.expected_receipt_hash();
        let profile = ArtifactSemanticProfile::from_packet(packet);
        packet.artifact_lineage_hash = profile.expected_lineage_hash();
        packet
    }

    #[test]
    fn verification_receipt_binds_semantic_payload_and_verdict() {
        let record = VerificationRecord::from_packet(valid_packet());
        let receipt = record.receipt();

        assert!(record.is_valid());
        assert!(receipt.is_self_consistent());
        assert!(receipt.is_valid_for(&record));
        assert_eq!(receipt.verdict, VerificationDecision::Accepted);
        assert_eq!(receipt.request_hash, record.request.contract_hash());
        assert_eq!(receipt.payload_hash, verification_payload_hash(&record));
        assert_eq!(receipt.submission().gate, GateId::Verification);
        assert!(receipt.submission().passed);
    }

    #[test]
    fn verification_receipt_rejects_tampered_payload() {
        let record = VerificationRecord::from_packet(valid_packet());
        let mut receipt = record.receipt();
        receipt.semantic_check_hash ^= 1;

        assert!(!receipt.is_self_consistent());
        assert!(!receipt.is_valid_for(&record));
        assert!(!receipt.submission().passed);
    }

    #[test]
    fn verification_receipt_records_rejected_profiles_without_passing_gate() {
        let mut packet = valid_packet();
        packet.artifact_receipt_hash ^= 1;

        let record = VerificationRecord::from_packet(packet);
        let receipt = record.receipt();

        assert_eq!(record.decision(), VerificationDecision::Rejected);
        assert!(receipt.is_self_consistent());
        assert!(receipt.is_valid_for(&record));
        assert_eq!(receipt.verdict, VerificationDecision::Rejected);
        assert_ne!(receipt.defect_mask, DEFECT_NONE);
        assert!(!receipt.submission().passed);
    }
}
