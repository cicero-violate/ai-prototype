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

use std::fmt;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

use crate::capability::{EvidenceProducer, EvidenceSubmission, PacketEffect};
use crate::kernel::{mix, Evidence, GateId, Packet};

pub const VERIFICATION_PROOF_SCHEMA_VERSION: u64 = 1;
pub const VERIFICATION_PROOF_RECORD: u64 = 61;

pub const PROOF_FLAG_RECEIPT_VERIFIED: u64 = 1 << 0;
pub const PROOF_FLAG_TAMPER_REJECTED: u64 = 1 << 1;
pub const PROOF_FLAG_PROVENANCE_VERIFIED: u64 = 1 << 2;
pub const PROOF_FLAG_PHASE_VERIFIED: u64 = 1 << 3;
pub const PROOF_FLAGS_REQUIRED: u64 = PROOF_FLAG_RECEIPT_VERIFIED
    | PROOF_FLAG_TAMPER_REJECTED
    | PROOF_FLAG_PROVENANCE_VERIFIED
    | PROOF_FLAG_PHASE_VERIFIED;

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
#[repr(u8)]
pub enum ProofSubjectKind {
    ArtifactEffect = 1,
    ProcessEffect = 2,
    LlmEffect = 3,
    SemanticVerification = 4,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VerificationProofRecord {
    pub subject: ProofSubjectKind,
    pub proof_line_hash: u64,
    pub receipt_core_hash: u64,
    pub receipt_hash: u64,
    pub receipt_event_seq: u64,
    pub proof_event_seq: u64,
    pub receipt_event_hash: u64,
    pub verifier_context_hash: u64,
    pub proof_flags: u64,
    pub provider_proof_hash: u64,
    pub record_hash: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VerificationProofBinding {
    pub subject: ProofSubjectKind,
    pub receipt_core_hash: u64,
    pub receipt_hash: u64,
    pub receipt_event_seq: u64,
    pub receipt_event_hash: u64,
    pub provider_proof_hash: u64,
}

impl VerificationProofBinding {
    pub fn new(
        subject: ProofSubjectKind,
        receipt_core_hash: u64,
        receipt_hash: u64,
        receipt_event_seq: u64,
        receipt_event_hash: u64,
        provider_proof_hash: u64,
    ) -> Option<Self> {
        let binding = Self {
            subject,
            receipt_core_hash,
            receipt_hash,
            receipt_event_seq,
            receipt_event_hash,
            provider_proof_hash,
        };
        binding.is_valid().then_some(binding)
    }

    pub fn is_valid(self) -> bool {
        self.receipt_core_hash != 0
            && self.receipt_hash != 0
            && self.receipt_event_seq != 0
            && self.receipt_event_hash != 0
            && self.provider_proof_hash != 0
    }
}

impl VerificationProofRecord {
    pub fn new(
        subject: ProofSubjectKind,
        proof_line_hash: u64,
        receipt_core_hash: u64,
        receipt_hash: u64,
        receipt_event_seq: u64,
        proof_event_seq: u64,
        receipt_event_hash: u64,
        verifier_context_hash: u64,
        proof_flags: u64,
        provider_proof_hash: u64,
    ) -> Option<Self> {
        let mut record = Self {
            subject,
            proof_line_hash,
            receipt_core_hash,
            receipt_hash,
            receipt_event_seq,
            proof_event_seq,
            receipt_event_hash,
            verifier_context_hash,
            proof_flags,
            provider_proof_hash,
            record_hash: 0,
        };
        record.record_hash = record.expected_record_hash();
        record.is_valid().then_some(record)
    }

    pub fn is_valid(self) -> bool {
        self.proof_line_hash != 0
            && self.receipt_core_hash != 0
            && self.receipt_hash != 0
            && self.receipt_event_seq != 0
            && self.proof_event_seq != 0
            && self.proof_event_seq > self.receipt_event_seq
            && self.receipt_event_hash != 0
            && self.verifier_context_hash != 0
            && (self.proof_flags & PROOF_FLAGS_REQUIRED) == PROOF_FLAGS_REQUIRED
            && self.provider_proof_hash != 0
            && self.record_hash != 0
            && self.record_hash == self.expected_record_hash()
    }

    pub fn expected_record_hash(self) -> u64 {
        let mut h = 0x5650_524f_4f46_0001u64;
        h = mix(h, self.subject as u64);
        h = mix(h, self.proof_line_hash);
        h = mix(h, self.receipt_core_hash);
        h = mix(h, self.receipt_hash);
        h = mix(h, self.receipt_event_seq);
        h = mix(h, self.proof_event_seq);
        h = mix(h, self.receipt_event_hash);
        h = mix(h, self.verifier_context_hash);
        h = mix(h, self.proof_flags);
        h = mix(h, self.provider_proof_hash);
        h.max(1)
    }

    pub fn matches_receipt_binding(
        self,
        receipt_core_hash: u64,
        receipt_hash: u64,
        receipt_event_seq: u64,
        receipt_event_hash: u64,
        provider_proof_hash: u64,
    ) -> bool {
        self.is_valid()
            && self.receipt_core_hash == receipt_core_hash
            && self.receipt_hash == receipt_hash
            && self.receipt_event_seq == receipt_event_seq
            && self.receipt_event_hash == receipt_event_hash
            && self.provider_proof_hash == provider_proof_hash
    }

    pub fn binding(self) -> Option<VerificationProofBinding> {
        VerificationProofBinding::new(
            self.subject,
            self.receipt_core_hash,
            self.receipt_hash,
            self.receipt_event_seq,
            self.receipt_event_hash,
            self.provider_proof_hash,
        )
    }

    pub fn matches_binding(self, binding: VerificationProofBinding) -> bool {
        binding.is_valid()
            && self.subject == binding.subject
            && self.matches_receipt_binding(
                binding.receipt_core_hash,
                binding.receipt_hash,
                binding.receipt_event_seq,
                binding.receipt_event_hash,
                binding.provider_proof_hash,
            )
    }
}

#[derive(Debug)]
pub enum VerificationProofError {
    Io(std::io::Error),
    InvalidRecord,
    InvalidProof,
}

impl fmt::Display for VerificationProofError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(err) => write!(f, "verification proof io failed: {err}"),
            Self::InvalidRecord => write!(f, "verification proof record is invalid"),
            Self::InvalidProof => write!(f, "verification proof is invalid"),
        }
    }
}

impl std::error::Error for VerificationProofError {}

impl From<std::io::Error> for VerificationProofError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
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

pub fn append_verification_proof_record_ndjson(
    path: impl AsRef<Path>,
    record: &VerificationProofRecord,
) -> Result<(), VerificationProofError> {
    if !record.is_valid() {
        return Err(VerificationProofError::InvalidProof);
    }

    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }

    {
        let mut file = OpenOptions::new().create(true).append(true).open(path)?;
        writeln!(file, "{}", encode_verification_proof_record_ndjson(*record))?;
        file.sync_all()?;
    }

    sync_parent_dir(path)
}

pub fn load_verification_proof_records_ndjson(
    path: impl AsRef<Path>,
) -> Result<Vec<VerificationProofRecord>, VerificationProofError> {
    let path = path.as_ref();
    if !path.exists() {
        return Ok(Vec::new());
    }

    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let fields = parse_u64_fields(&line)?;
        if fields.len() >= 2 && fields[1] == VERIFICATION_PROOF_RECORD {
            records.push(decode_verification_proof_record_fields(&fields)?);
        }
    }
    Ok(records)
}

pub fn verify_verification_proof_records(
    records: &[VerificationProofRecord],
) -> Result<usize, VerificationProofError> {
    if records.is_empty() {
        return Err(VerificationProofError::InvalidProof);
    }

    let mut seen_receipt_hashes = Vec::new();
    for record in records {
        if !record.is_valid() || seen_receipt_hashes.contains(&record.receipt_hash) {
            return Err(VerificationProofError::InvalidProof);
        }
        seen_receipt_hashes.push(record.receipt_hash);
    }

    Ok(records.len())
}

pub fn verify_verification_proof_record_bindings(
    records: &[VerificationProofRecord],
    bindings: &[VerificationProofBinding],
) -> Result<usize, VerificationProofError> {
    if records.is_empty() || records.len() != bindings.len() {
        return Err(VerificationProofError::InvalidProof);
    }

    let mut seen_receipt_hashes = Vec::new();
    for record in records {
        if !record.is_valid() || seen_receipt_hashes.contains(&record.receipt_hash) {
            return Err(VerificationProofError::InvalidProof);
        }

        if !bindings
            .iter()
            .copied()
            .any(|binding| record.matches_binding(binding))
        {
            return Err(VerificationProofError::InvalidProof);
        }

        seen_receipt_hashes.push(record.receipt_hash);
    }

    Ok(records.len())
}

pub fn verify_verification_proof_records_ndjson(
    path: impl AsRef<Path>,
) -> Result<usize, VerificationProofError> {
    let records = load_verification_proof_records_ndjson(path)?;
    verify_verification_proof_records(&records)
}

pub fn encode_verification_proof_record_ndjson(record: VerificationProofRecord) -> String {
    let fields = [
        VERIFICATION_PROOF_SCHEMA_VERSION,
        VERIFICATION_PROOF_RECORD,
        record.subject as u64,
        record.proof_line_hash,
        record.receipt_core_hash,
        record.receipt_hash,
        record.receipt_event_seq,
        record.proof_event_seq,
        record.receipt_event_hash,
        record.verifier_context_hash,
        record.proof_flags,
        record.provider_proof_hash,
        record.record_hash,
    ];
    let body = fields
        .iter()
        .map(u64::to_string)
        .collect::<Vec<_>>()
        .join(",");
    format!("[{body}]")
}

pub fn decode_verification_proof_record_ndjson(
    line: &str,
) -> Result<VerificationProofRecord, VerificationProofError> {
    decode_verification_proof_record_fields(&parse_u64_fields(line)?)
}

fn decode_verification_proof_record_fields(
    fields: &[u64],
) -> Result<VerificationProofRecord, VerificationProofError> {
    if fields.len() != 13
        || fields[0] != VERIFICATION_PROOF_SCHEMA_VERSION
        || fields[1] != VERIFICATION_PROOF_RECORD
    {
        return Err(VerificationProofError::InvalidRecord);
    }

    let subject = proof_subject_kind_from_u64(fields[2])?;
    let record = VerificationProofRecord {
        subject,
        proof_line_hash: fields[3],
        receipt_core_hash: fields[4],
        receipt_hash: fields[5],
        receipt_event_seq: fields[6],
        proof_event_seq: fields[7],
        receipt_event_hash: fields[8],
        verifier_context_hash: fields[9],
        proof_flags: fields[10],
        provider_proof_hash: fields[11],
        record_hash: fields[12],
    };
    record
        .is_valid()
        .then_some(record)
        .ok_or(VerificationProofError::InvalidProof)
}

fn proof_subject_kind_from_u64(value: u64) -> Result<ProofSubjectKind, VerificationProofError> {
    match value {
        1 => Ok(ProofSubjectKind::ArtifactEffect),
        2 => Ok(ProofSubjectKind::ProcessEffect),
        3 => Ok(ProofSubjectKind::LlmEffect),
        4 => Ok(ProofSubjectKind::SemanticVerification),
        _ => Err(VerificationProofError::InvalidRecord),
    }
}

fn parse_u64_fields(line: &str) -> Result<Vec<u64>, VerificationProofError> {
    let body = line
        .trim()
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
        .ok_or(VerificationProofError::InvalidRecord)?;
    if body.trim().is_empty() {
        return Ok(Vec::new());
    }
    body.split(',')
        .map(|raw| {
            raw.trim()
                .parse::<u64>()
                .map_err(|_| VerificationProofError::InvalidRecord)
        })
        .collect()
}

fn sync_parent_dir(path: &Path) -> Result<(), VerificationProofError> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            File::open(parent)?.sync_all()?;
        }
    }
    Ok(())
}