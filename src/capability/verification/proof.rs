//! Generic verification proof records and replay checks.
//!
//! This module owns the provider/tool/process proof spine:
//!
//! ```text
//! receipt binding -> VerificationProofRecord -> mixed NDJSON replay check
//! ```
//!
//! Keeping this separate from semantic artifact verification reduces coupling:
//! semantic verification can stay focused on artifact profiles while every
//! proof-producing effect shares one durable replay contract.

use std::fmt;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

use crate::codec::ndjson::{load_tlog_ndjson, TLOG_RECORD_EVENT, TLOG_SCHEMA_VERSION};
use crate::kernel::{mix, TLog};

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

    pub fn from_binding(
        binding: VerificationProofBinding,
        proof_line_hash: u64,
        proof_event_seq: u64,
        verifier_context_hash: u64,
        proof_flags: u64,
    ) -> Option<Self> {
        if !binding.is_valid() || proof_event_seq <= binding.receipt_event_seq {
            return None;
        }

        Self::new(
            binding.subject,
            proof_line_hash,
            binding.receipt_core_hash,
            binding.receipt_hash,
            binding.receipt_event_seq,
            proof_event_seq,
            binding.receipt_event_hash,
            verifier_context_hash,
            proof_flags,
            binding.provider_proof_hash,
        )
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

pub fn verify_verification_proof_record_replay(
    tlog: &TLog,
    records: &[VerificationProofRecord],
    bindings: &[VerificationProofBinding],
) -> Result<usize, VerificationProofError> {
    let verified = verify_verification_proof_record_bindings(records, bindings)?;

    for record in records {
        let matching_receipt_events = tlog
            .iter()
            .filter(|event| {
                event.seq == record.receipt_event_seq
                    && event.self_hash == record.receipt_event_hash
            })
            .count();
        if matching_receipt_events != 1 {
            return Err(VerificationProofError::InvalidProof);
        }
    }

    Ok(verified)
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

pub fn verify_verification_proof_record_order_ndjson(
    path: impl AsRef<Path>,
) -> Result<usize, VerificationProofError> {
    let path = path.as_ref();
    if !path.exists() {
        return Err(VerificationProofError::InvalidProof);
    }

    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut synthetic_seq = 0u64;
    let mut seen_receipt_hashes = Vec::new();
    let mut verified = 0usize;

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        let fields = parse_u64_fields(&line)?;
        if fields.len() < 2 {
            return Err(VerificationProofError::InvalidRecord);
        }

        match (fields[0], fields[1]) {
            (TLOG_SCHEMA_VERSION, TLOG_RECORD_EVENT) => {
                let event_seq = *fields.get(2).ok_or(VerificationProofError::InvalidRecord)?;
                if event_seq == 0 || event_seq <= synthetic_seq {
                    return Err(VerificationProofError::InvalidProof);
                }
                synthetic_seq = event_seq;
            }
            (VERIFICATION_PROOF_SCHEMA_VERSION, VERIFICATION_PROOF_RECORD) => {
                let record = decode_verification_proof_record_fields(&fields)?;
                let expected_proof_event_seq = synthetic_seq
                    .checked_add(1)
                    .filter(|seq| *seq != 0)
                    .ok_or(VerificationProofError::InvalidProof)?;
                if record.proof_event_seq != expected_proof_event_seq
                    || seen_receipt_hashes.contains(&record.receipt_hash)
                {
                    return Err(VerificationProofError::InvalidProof);
                }
                synthetic_seq = record.proof_event_seq;
                seen_receipt_hashes.push(record.receipt_hash);
                verified += 1;
            }
            _ => {}
        }
    }

    if verified == 0 {
        return Err(VerificationProofError::InvalidProof);
    }

    Ok(verified)
}

pub fn verify_verification_proof_record_replay_ndjson(
    path: impl AsRef<Path>,
    bindings: &[VerificationProofBinding],
) -> Result<usize, VerificationProofError> {
    let path = path.as_ref();
    let tlog = load_tlog_ndjson(path).map_err(|_| VerificationProofError::InvalidProof)?;
    let records = load_verification_proof_records_ndjson(path)?;
    let verified = verify_verification_proof_record_replay(&tlog, &records, bindings)?;
    if verify_verification_proof_record_order_ndjson(path)? != verified {
        return Err(VerificationProofError::InvalidProof);
    }
    Ok(verified)
}

pub fn verify_verification_proof_records_ndjson(
    path: impl AsRef<Path>,
) -> Result<usize, VerificationProofError> {
    let path = path.as_ref();
    let records = load_verification_proof_records_ndjson(path)?;
    let verified = verify_verification_proof_records(&records)?;
    if verify_verification_proof_record_order_ndjson(path)? != verified {
        return Err(VerificationProofError::InvalidProof);
    }
    Ok(verified)
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