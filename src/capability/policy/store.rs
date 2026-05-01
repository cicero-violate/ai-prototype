//! Append-only policy store.

use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

use crate::capability::learning::{
    PolicyPromotion, POLICY_FEEDBACK_HASH, POLICY_PROMOTION_SOURCE_SEQ,
};
use crate::capability::verification::{
    GenericVerificationProofSubject, ProofSubjectKind, VerificationProofBinding,
    VerificationProofRecord, PROOF_FLAGS_REQUIRED,
};
use crate::kernel::mix;

const POLICY_SCHEMA_VERSION: u64 = 1;
const POLICY_RECORD_ENTRY: u64 = 1;
const POLICY_KEY_PROMOTION_SOURCE_SEQ: u64 = 1;
const POLICY_KEY_FEEDBACK_HASH: u64 = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PolicyEntry {
    pub version: u64,
    pub key: &'static str,
    pub value: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PolicyProofReceipt {
    pub entry: PolicyEntry,
    pub policy_store_hash: u64,
    pub receipt_event_seq: u64,
    pub receipt_event_hash: u64,
    pub receipt_hash: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PolicyStoreError {
    InvalidPromotion,
    NonMonotonicVersion,
    UnknownPolicyKey,
    PolicyIo,
    InvalidPolicyRecord,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PolicyStore {
    entries: Vec<PolicyEntry>,
}

impl PolicyStore {
    pub fn append(&mut self, entry: PolicyEntry) {
        assert!(
            entry.version > self.latest_version(),
            "policy versions must increase monotonically"
        );
        self.entries.push(entry);
    }

    pub fn try_append(&mut self, entry: PolicyEntry) -> Result<(), PolicyStoreError> {
        if entry.version <= self.latest_version() {
            return Err(PolicyStoreError::NonMonotonicVersion);
        }
        key_to_id(entry.key)?;
        self.entries.push(entry);
        Ok(())
    }

    pub fn append_durable(
        &mut self,
        path: impl AsRef<Path>,
        entry: PolicyEntry,
    ) -> Result<&PolicyEntry, PolicyStoreError> {
        if entry.version <= self.latest_version() {
            return Err(PolicyStoreError::NonMonotonicVersion);
        }
        key_to_id(entry.key)?;
        append_policy_ndjson(path, &entry)?;
        self.entries.push(entry);
        self.entries
            .last()
            .ok_or(PolicyStoreError::InvalidPolicyRecord)
    }

    pub fn promote(
        &mut self,
        promotion: PolicyPromotion,
    ) -> Result<&PolicyEntry, PolicyStoreError> {
        if !promotion.is_valid() {
            return Err(PolicyStoreError::InvalidPromotion);
        }

        self.try_append(PolicyEntry {
            version: promotion.promoted_policy_version,
            key: POLICY_PROMOTION_SOURCE_SEQ,
            value: promotion.source_seq,
        })?;

        self.entries.last().ok_or(PolicyStoreError::InvalidPromotion)
    }

    pub fn promote_feedback(
        &mut self,
        promotion: PolicyPromotion,
    ) -> Result<&PolicyEntry, PolicyStoreError> {
        if !promotion.is_valid() {
            return Err(PolicyStoreError::InvalidPromotion);
        }

        self.try_append(PolicyEntry {
            version: promotion.promoted_policy_version,
            key: POLICY_FEEDBACK_HASH,
            value: promotion.promoted_policy_hash,
        })?;

        self.entries.last().ok_or(PolicyStoreError::InvalidPromotion)
    }

    pub fn promote_durable(
        &mut self,
        path: impl AsRef<Path>,
        promotion: PolicyPromotion,
    ) -> Result<&PolicyEntry, PolicyStoreError> {
        if !promotion.is_valid() {
            return Err(PolicyStoreError::InvalidPromotion);
        }

        self.append_durable(
            path,
            PolicyEntry {
                version: promotion.promoted_policy_version,
                key: POLICY_PROMOTION_SOURCE_SEQ,
                value: promotion.source_seq,
            },
        )
    }

    pub fn promote_feedback_durable(
        &mut self,
        path: impl AsRef<Path>,
        promotion: PolicyPromotion,
    ) -> Result<&PolicyEntry, PolicyStoreError> {
        if !promotion.is_valid() {
            return Err(PolicyStoreError::InvalidPromotion);
        }

        self.append_durable(
            path,
            PolicyEntry {
                version: promotion.promoted_policy_version,
                key: POLICY_FEEDBACK_HASH,
                value: promotion.promoted_policy_hash,
            },
        )
    }

    pub fn load_ndjson(path: impl AsRef<Path>) -> Result<Self, PolicyStoreError> {
        let path = path.as_ref();
        if !path.exists() {
            return Ok(Self::default());
        }

        let file = File::open(path).map_err(|_| PolicyStoreError::PolicyIo)?;
        let reader = BufReader::new(file);
        let mut store = Self::default();

        for line in reader.lines() {
            let line = line.map_err(|_| PolicyStoreError::PolicyIo)?;
            if line.trim().is_empty() {
                continue;
            }
            store.try_append(decode_policy_entry_ndjson(&line)?)?;
        }

        Ok(store)
    }

    pub fn latest(&self, key: &str) -> Option<&PolicyEntry> {
        self.entries.iter().rev().find(|entry| entry.key == key)
    }

    pub fn latest_version(&self) -> u64 {
        self.entries
            .iter()
            .map(|entry| entry.version)
            .max()
            .unwrap_or(0)
    }

    pub fn latest_value(&self, key: &str) -> Option<u64> {
        self.latest(key).map(|entry| entry.value)
    }

    pub fn feedback_hash(&self) -> u64 {
        self.latest_value(POLICY_FEEDBACK_HASH).unwrap_or(0)
    }

    pub fn fingerprint(&self) -> u64 {
        let mut h = 0xcbf2_9ce4_8422_2325u64;

        for entry in &self.entries {
            let key_id = key_to_id(entry.key).expect("validated policy key");
            h = mix(h, entry.version);
            h = mix(h, key_id);
            h = mix(h, entry.value);
        }

        h.max(1)
    }

    pub fn entries(&self) -> &[PolicyEntry] {
        &self.entries
    }
}

impl PolicyProofReceipt {
    pub fn new(
        entry: PolicyEntry,
        policy_store_hash: u64,
        receipt_event_seq: u64,
        receipt_event_hash: u64,
    ) -> Option<Self> {
        let mut receipt = Self {
            entry,
            policy_store_hash,
            receipt_event_seq,
            receipt_event_hash,
            receipt_hash: 0,
        };
        receipt.receipt_hash = receipt.expected_receipt_hash()?;
        receipt.is_valid().then_some(receipt)
    }

    pub fn is_valid(self) -> bool {
        self.entry_hash().is_some()
            && self.policy_store_hash != 0
            && self.receipt_event_seq != 0
            && self.receipt_event_hash != 0
            && self.receipt_hash != 0
            && self.receipt_hash == self.expected_receipt_hash().unwrap_or(0)
    }

    pub fn entry_hash(self) -> Option<u64> {
        let mut h = 0x504f_4c49_4359_454eu64;
        h = mix(h, self.entry.version);
        h = mix(h, key_to_id(self.entry.key).ok()?);
        h = mix(h, self.entry.value);
        Some(h.max(1))
    }

    pub fn receipt_core_hash(self) -> Option<u64> {
        let mut h = 0x504f_4c49_4359_434fu64;
        h = mix(h, self.entry_hash()?);
        h = mix(h, self.policy_store_hash);
        h = mix(h, self.receipt_event_seq);
        h = mix(h, self.receipt_event_hash);
        Some(h.max(1))
    }

    pub fn expected_receipt_hash(self) -> Option<u64> {
        let mut h = 0x504f_4c49_4359_5243u64;
        h = mix(h, self.receipt_core_hash()?);
        h = mix(h, self.entry_hash()?);
        h = mix(h, self.policy_store_hash);
        Some(h.max(1))
    }

    pub fn verifier_context_hash(self) -> Option<u64> {
        let mut h = 0x504f_4c49_4359_4354u64;
        h = mix(h, ProofSubjectKind::PolicyEffect as u64);
        h = mix(h, self.entry_hash()?);
        h = mix(h, self.policy_store_hash);
        Some(h.max(1))
    }

    pub fn provider_proof_hash(self, proof_event_seq: u64) -> Option<u64> {
        if !self.is_valid() || proof_event_seq <= self.receipt_event_seq {
            return None;
        }

        let mut h = 0x504f_4c49_4359_5052u64;
        h = mix(h, self.receipt_core_hash()?);
        h = mix(h, self.receipt_hash);
        h = mix(h, self.receipt_event_hash);
        h = mix(h, proof_event_seq);
        h = mix(h, self.entry_hash()?);
        Some(h.max(1))
    }

    pub fn proof_line_hash(self, proof_event_seq: u64) -> Option<u64> {
        let mut h = 0x504f_4c49_4359_4c4eu64;
        h = mix(h, self.receipt_core_hash()?);
        h = mix(h, self.receipt_hash);
        h = mix(h, self.receipt_event_seq);
        h = mix(h, proof_event_seq);
        h = mix(h, self.provider_proof_hash(proof_event_seq)?);
        Some(h.max(1))
    }

    pub fn verification_proof_binding(
        self,
        proof_event_seq: u64,
    ) -> Option<VerificationProofBinding> {
        VerificationProofBinding::new(
            ProofSubjectKind::PolicyEffect,
            self.receipt_core_hash()?,
            self.receipt_hash,
            self.receipt_event_seq,
            self.receipt_event_hash,
            self.provider_proof_hash(proof_event_seq)?,
        )
    }

    pub fn verification_proof_subject(
        self,
        proof_event_seq: u64,
    ) -> Option<GenericVerificationProofSubject> {
        GenericVerificationProofSubject::from_binding(
            self.verification_proof_binding(proof_event_seq)?,
            self.proof_line_hash(proof_event_seq)?,
            proof_event_seq,
            self.verifier_context_hash()?,
            PROOF_FLAGS_REQUIRED,
        )
    }

    pub fn to_verification_proof_record(
        self,
        proof_event_seq: u64,
    ) -> Option<VerificationProofRecord> {
        VerificationProofRecord::from_subject(self.verification_proof_subject(proof_event_seq)?)
    }
}

fn append_policy_ndjson(
    path: impl AsRef<Path>,
    entry: &PolicyEntry,
) -> Result<(), PolicyStoreError> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|_| PolicyStoreError::PolicyIo)?;
    writeln!(file, "{}", encode_policy_entry_ndjson(entry))
        .map_err(|_| PolicyStoreError::PolicyIo)?;
    file.sync_all().map_err(|_| PolicyStoreError::PolicyIo)
}

fn encode_policy_entry_ndjson(entry: &PolicyEntry) -> String {
    let key = key_to_id(entry.key).expect("validated policy key");
    format!(
        "[{POLICY_SCHEMA_VERSION},{POLICY_RECORD_ENTRY},{},{},{}]",
        entry.version, key, entry.value
    )
}

fn decode_policy_entry_ndjson(line: &str) -> Result<PolicyEntry, PolicyStoreError> {
    let trimmed = line.trim();
    let body = trimmed
        .strip_prefix('[')
        .and_then(|v| v.strip_suffix(']'))
        .ok_or(PolicyStoreError::InvalidPolicyRecord)?;
    let fields = body
        .split(',')
        .map(|raw| {
            raw.trim()
                .parse::<u64>()
                .map_err(|_| PolicyStoreError::InvalidPolicyRecord)
        })
        .collect::<Result<Vec<_>, _>>()?;

    if fields.len() != 5
        || fields[0] != POLICY_SCHEMA_VERSION
        || fields[1] != POLICY_RECORD_ENTRY
    {
        return Err(PolicyStoreError::InvalidPolicyRecord);
    }

    Ok(PolicyEntry {
        version: fields[2],
        key: key_from_id(fields[3])?,
        value: fields[4],
    })
}

fn key_to_id(key: &str) -> Result<u64, PolicyStoreError> {
    match key {
        POLICY_PROMOTION_SOURCE_SEQ => Ok(POLICY_KEY_PROMOTION_SOURCE_SEQ),
        POLICY_FEEDBACK_HASH => Ok(POLICY_KEY_FEEDBACK_HASH),
        _ => Err(PolicyStoreError::UnknownPolicyKey),
    }
}

fn key_from_id(id: u64) -> Result<&'static str, PolicyStoreError> {
    match id {
        POLICY_KEY_PROMOTION_SOURCE_SEQ => Ok(POLICY_PROMOTION_SOURCE_SEQ),
        POLICY_KEY_FEEDBACK_HASH => Ok(POLICY_FEEDBACK_HASH),
        _ => Err(PolicyStoreError::UnknownPolicyKey),
    }
}

