//! Append-only policy store.

use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

use crate::capability::verification::{
    CanonicalEffect, CanonicalEffectProof, CanonicalEffectReceipt, ProofSubjectKind,
    VerificationProofBinding, VerificationProofRecord, PROOF_FLAGS_REQUIRED,
};
use crate::kernel::mix;

const POLICY_SCHEMA_VERSION: u64 = 1;
const POLICY_RECORD_ENTRY: u64 = 1;
const POLICY_KEY_PROMOTION_SOURCE_SEQ: u64 = 1;
const POLICY_KEY_FEEDBACK_HASH: u64 = 2;

pub const POLICY_PROMOTION_SOURCE_SEQ: &str = "learning.policy_promotion.source_seq";
pub const POLICY_FEEDBACK_HASH: &str = "learning.policy_feedback.hash";

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PolicyLookupReceipt {
    pub key_hash: u64,
    pub requested_key_id: u64,
    pub found_version: u64,
    pub found_value: u64,
    pub policy_store_hash: u64,
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

    pub fn lookup_with_receipt(
        &self,
        key: &'static str,
    ) -> (Option<PolicyEntry>, PolicyLookupReceipt) {
        let entry = self.latest(key).copied();
        let receipt = PolicyLookupReceipt::new(key, entry, self.fingerprint());
        (entry, receipt)
    }

    pub fn feedback_lookup_with_receipt(&self) -> (Option<PolicyEntry>, PolicyLookupReceipt) {
        self.lookup_with_receipt(POLICY_FEEDBACK_HASH)
    }

    pub fn fingerprint(&self) -> u64 {
        let mut h = 0xcbf2_9ce4_8422_2325u64;

        for entry in &self.entries {
            let Ok(key_id) = key_to_id(entry.key) else {
                return 0;
            };
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

impl PolicyLookupReceipt {
    pub fn new(key: &'static str, entry: Option<PolicyEntry>, policy_store_hash: u64) -> Self {
        let requested_key_id = key_to_id(key).unwrap_or(0);
        let key_hash = policy_lookup_key_hash(requested_key_id);
        let (found_version, found_value) = entry
            .filter(|entry| key_to_id(entry.key).ok() == Some(requested_key_id))
            .map(|entry| (entry.version, entry.value))
            .unwrap_or((0, 0));
        let mut receipt = Self {
            key_hash,
            requested_key_id,
            found_version,
            found_value,
            policy_store_hash,
            receipt_hash: 0,
        };
        receipt.receipt_hash = receipt.expected_receipt_hash();
        receipt
    }

    pub fn is_hit(self) -> bool {
        self.is_valid() && self.found_version != 0 && self.found_value != 0
    }

    pub fn is_valid(self) -> bool {
        self.requested_key_id != 0
            && self.key_hash == policy_lookup_key_hash(self.requested_key_id)
            && self.policy_store_hash != 0
            && self.receipt_hash != 0
            && self.receipt_hash == self.expected_receipt_hash()
    }

    pub fn is_valid_for(self, store: &PolicyStore, key: &'static str) -> bool {
        let expected =
            PolicyLookupReceipt::new(key, store.latest(key).copied(), store.fingerprint());
        self == expected && self.is_valid()
    }

    pub fn expected_receipt_hash(self) -> u64 {
        if self.requested_key_id == 0 || self.key_hash == 0 || self.policy_store_hash == 0 {
            return 0;
        }
        let mut h = 0x504f_4c49_4359_4c4bu64;
        h = mix(h, self.key_hash);
        h = mix(h, self.requested_key_id);
        h = mix(h, self.found_version);
        h = mix(h, self.found_value);
        h = mix(h, self.policy_store_hash);
        h.max(1)
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
        Some(fold_policy_proof_hash(
            0x504f_4c49_4359_434fu64,
            &[
                self.entry_hash()?,
                self.policy_store_hash,
                self.receipt_event_seq,
                self.receipt_event_hash,
            ],
        ))
    }

    pub fn expected_receipt_hash(self) -> Option<u64> {
        let mut h = 0x504f_4c49_4359_5243u64;
        h = mix(h, self.receipt_core_hash()?);
        h = mix(h, self.entry_hash()?);
        h = mix(h, self.policy_store_hash);
        Some(h.max(1))
    }

    pub fn verifier_context_hash(self) -> Option<u64> {
        Some(fold_policy_proof_hash(
            0x504f_4c49_4359_4354u64,
            &[
                ProofSubjectKind::PolicyEffect as u64,
                self.entry_hash()?,
                self.policy_store_hash,
            ],
        ))
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

    pub fn canonical_authority_hash(self) -> Option<u64> {
        if !self.is_valid() {
            return None;
        }
        let mut h = 0x504f_4c49_4359_4155u64;
        h = mix(h, self.policy_store_hash);
        Some(h.max(1))
    }

    pub fn canonical_request_hash(self) -> Option<u64> {
        if !self.is_valid() {
            return None;
        }
        let mut h = 0x504f_4c49_4359_5251u64;
        h = mix(h, self.entry_hash()?);
        Some(h.max(1))
    }

    pub fn canonical_effect(self) -> Option<CanonicalEffect> {
        CanonicalEffect::policy(self.entry_hash()?, self.policy_store_hash)
    }

    pub fn canonical_effect_receipt(self) -> Option<CanonicalEffectReceipt> {
        CanonicalEffectReceipt::new_unbound(
            ProofSubjectKind::PolicyEffect,
            self.canonical_effect()?,
            self.canonical_authority_hash()?,
            self.canonical_request_hash()?,
            self.receipt_event_seq,
            self.receipt_event_hash,
        )
    }

    pub fn to_canonical_effect_proof(
        self,
        proof_event_seq: u64,
    ) -> Option<(CanonicalEffectReceipt, CanonicalEffectProof)> {
        CanonicalEffectProof::finalize(
            self.canonical_effect_receipt()?,
            proof_event_seq,
            self.verifier_context_hash()?,
            PROOF_FLAGS_REQUIRED,
            self.provider_proof_hash(proof_event_seq)?,
        )
    }

    pub fn proof_line_hash(self, proof_event_seq: u64) -> Option<u64> {
        Some(
            self.to_canonical_effect_proof(proof_event_seq)?
                .1
                .proof_line_hash,
        )
    }

    pub fn verification_proof_binding(
        self,
        proof_event_seq: u64,
    ) -> Option<VerificationProofBinding> {
        self.to_canonical_effect_proof(proof_event_seq)?
            .1
            .verification_proof_binding()
    }

    pub fn to_verification_proof_record(
        self,
        proof_event_seq: u64,
    ) -> Option<VerificationProofRecord> {
        self.to_canonical_effect_proof(proof_event_seq)?
            .1
            .to_verification_proof_record()
    }
}

fn fold_policy_proof_hash(seed: u64, fields: &[u64]) -> u64 {
    fields.iter().copied().fold(seed, mix).max(1)
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
    writeln!(file, "{}", encode_policy_entry_ndjson(entry)?)
        .map_err(|_| PolicyStoreError::PolicyIo)?;
    file.sync_all().map_err(|_| PolicyStoreError::PolicyIo)
}

fn encode_policy_entry_ndjson(entry: &PolicyEntry) -> Result<String, PolicyStoreError> {
    let key = key_to_id(entry.key)?;
    Ok(format!(
        "[{POLICY_SCHEMA_VERSION},{POLICY_RECORD_ENTRY},{},{},{}]",
        entry.version, key, entry.value
    ))
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

    if fields.len() != 5 || fields[0] != POLICY_SCHEMA_VERSION || fields[1] != POLICY_RECORD_ENTRY {
        return Err(PolicyStoreError::InvalidPolicyRecord);
    }

    Ok(PolicyEntry {
        version: fields[2],
        key: key_from_id(fields[3])?,
        value: fields[4],
    })
}

fn policy_lookup_key_hash(key_id: u64) -> u64 {
    if key_id == 0 {
        return 0;
    }
    let mut h = 0x504f_4c49_4359_4b45u64;
    h = mix(h, key_id);
    h.max(1)
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

#[cfg(test)]
mod lookup_receipt_tests {
    use super::*;

    fn store_with_feedback() -> PolicyStore {
        let mut store = PolicyStore::default();
        store
            .try_append(PolicyEntry {
                version: 1,
                key: POLICY_FEEDBACK_HASH,
                value: 0xfeed,
            })
            .expect("test setup should succeed");
        store
    }

    #[test]
    fn policy_lookup_receipt_binds_feedback_hit() {
        let store = store_with_feedback();
        let (entry, receipt) = store.feedback_lookup_with_receipt();

        assert_eq!(entry.expect("test value should be present").value, 0xfeed);
        assert_eq!(receipt.found_version, 1);
        assert_eq!(receipt.found_value, 0xfeed);
        assert_eq!(receipt.policy_store_hash, store.fingerprint());
        assert!(receipt.is_hit());
        assert!(receipt.is_valid_for(&store, POLICY_FEEDBACK_HASH));
    }

    #[test]
    fn policy_lookup_receipt_records_deterministic_miss() {
        let store = PolicyStore::default();
        let (entry, receipt) = store.feedback_lookup_with_receipt();

        assert!(entry.is_none());
        assert_eq!(receipt.found_version, 0);
        assert_eq!(receipt.found_value, 0);
        assert!(receipt.is_valid());
        assert!(!receipt.is_hit());
        assert!(receipt.is_valid_for(&store, POLICY_FEEDBACK_HASH));
    }

    #[test]
    fn policy_lookup_receipt_rejects_tampered_store_or_value() {
        let store = store_with_feedback();
        let (_, mut receipt) = store.feedback_lookup_with_receipt();
        receipt.found_value ^= 1;

        assert!(!receipt.is_valid());
        assert!(!receipt.is_valid_for(&store, POLICY_FEEDBACK_HASH));

        let mut changed_store = store.clone();
        changed_store
            .try_append(PolicyEntry {
                version: 2,
                key: POLICY_PROMOTION_SOURCE_SEQ,
                value: 7,
            })
            .expect("test setup should succeed");
        let (_, original_receipt) = store.feedback_lookup_with_receipt();
        assert!(!original_receipt.is_valid_for(&changed_store, POLICY_FEEDBACK_HASH));
    }

    #[test]
    fn policy_proof_hash_helper_preserves_receipt_and_verifier_boundaries() {
        let mut store = PolicyStore::default();
        store
            .try_append(PolicyEntry {
                version: 1,
                key: POLICY_FEEDBACK_HASH,
                value: 0xfeed,
            })
            .expect("test setup should succeed");
        let entry = store
            .latest(POLICY_FEEDBACK_HASH)
            .copied()
            .expect("test setup should succeed");
        let policy_store_hash = store.fingerprint();
        let receipt = PolicyProofReceipt::new(entry, policy_store_hash, 7, 0xabc)
            .expect("test setup should succeed");

        let entry_hash = receipt.entry_hash().expect("test setup should succeed");
        let receipt_core_hash = receipt
            .receipt_core_hash()
            .expect("test setup should succeed");
        let expected_receipt_hash = receipt
            .expected_receipt_hash()
            .expect("test setup should succeed");
        let verifier_context_hash = receipt
            .verifier_context_hash()
            .expect("test setup should succeed");
        let canonical_authority_hash = receipt
            .canonical_authority_hash()
            .expect("test setup should succeed");
        let canonical_request_hash = receipt
            .canonical_request_hash()
            .expect("test setup should succeed");

        for value in [
            entry_hash,
            receipt_core_hash,
            expected_receipt_hash,
            verifier_context_hash,
            canonical_authority_hash,
            canonical_request_hash,
        ] {
            assert_ne!(value, 0);
        }
        assert_ne!(entry_hash, receipt_core_hash);
        assert_ne!(entry_hash, expected_receipt_hash);
        assert_ne!(entry_hash, verifier_context_hash);
        assert_ne!(entry_hash, canonical_authority_hash);
        assert_ne!(entry_hash, canonical_request_hash);
        assert_ne!(receipt_core_hash, expected_receipt_hash);
        assert_ne!(receipt_core_hash, verifier_context_hash);
        assert_ne!(receipt_core_hash, canonical_authority_hash);
        assert_ne!(receipt_core_hash, canonical_request_hash);
        assert_ne!(expected_receipt_hash, verifier_context_hash);
        assert_ne!(expected_receipt_hash, canonical_authority_hash);
        assert_ne!(expected_receipt_hash, canonical_request_hash);
        assert_ne!(verifier_context_hash, canonical_authority_hash);
        assert_ne!(verifier_context_hash, canonical_request_hash);
        assert_ne!(canonical_authority_hash, canonical_request_hash);

        assert_eq!(
            receipt_core_hash,
            fold_policy_proof_hash(
                0x504f_4c49_4359_434fu64,
                &[
                    entry_hash,
                    policy_store_hash,
                    receipt.receipt_event_seq,
                    receipt.receipt_event_hash,
                ],
            )
        );
        assert_eq!(
            verifier_context_hash,
            fold_policy_proof_hash(
                0x504f_4c49_4359_4354u64,
                &[
                    ProofSubjectKind::PolicyEffect as u64,
                    entry_hash,
                    policy_store_hash,
                ],
            )
        );
        assert_eq!(receipt.receipt_hash, expected_receipt_hash);
        assert_eq!(
            PolicyProofReceipt::new(entry, policy_store_hash, 7, 0xabc)
                .expect("test value should be present")
                .receipt_core_hash(),
            Some(receipt_core_hash)
        );
        assert_eq!(
            PolicyProofReceipt::new(entry, policy_store_hash, 7, 0xabc)
                .expect("test value should be present")
                .verifier_context_hash(),
            Some(verifier_context_hash)
        );

        let changed_store_hash = PolicyProofReceipt::new(entry, policy_store_hash ^ 1, 7, 0xabc)
            .expect("test value should be present")
            .receipt_core_hash();
        assert_ne!(changed_store_hash, Some(receipt_core_hash));
        assert_ne!(
            PolicyProofReceipt::new(entry, policy_store_hash ^ 1, 7, 0xabc)
                .expect("test value should be present")
                .verifier_context_hash(),
            Some(verifier_context_hash)
        );
        assert_ne!(
            PolicyProofReceipt::new(entry, policy_store_hash, 8, 0xabc)
                .expect("test value should be present")
                .receipt_core_hash(),
            Some(receipt_core_hash)
        );
        assert_eq!(
            PolicyProofReceipt::new(entry, policy_store_hash, 8, 0xabc)
                .expect("test value should be present")
                .verifier_context_hash(),
            Some(verifier_context_hash)
        );
        assert_ne!(
            PolicyProofReceipt::new(entry, policy_store_hash, 7, 0xabd)
                .expect("test value should be present")
                .receipt_core_hash(),
            Some(receipt_core_hash)
        );
        assert_eq!(
            PolicyProofReceipt::new(entry, policy_store_hash, 7, 0xabd)
                .expect("test value should be present")
                .verifier_context_hash(),
            Some(verifier_context_hash)
        );

        let changed_entry = PolicyEntry {
            version: 2,
            key: POLICY_FEEDBACK_HASH,
            value: 0xfeed,
        };
        assert_ne!(
            PolicyProofReceipt::new(changed_entry, policy_store_hash, 7, 0xabc)
                .expect("test value should be present")
                .receipt_core_hash(),
            Some(receipt_core_hash)
        );
        assert_ne!(
            PolicyProofReceipt::new(changed_entry, policy_store_hash, 7, 0xabc)
                .expect("test value should be present")
                .verifier_context_hash(),
            Some(verifier_context_hash)
        );

        let (effect_receipt, effect_proof) = receipt
            .to_canonical_effect_proof(8)
            .expect("test setup should succeed");
        assert!(effect_receipt.is_valid());
        assert!(effect_proof.is_valid());
        assert_eq!(effect_proof.verifier_context_hash, verifier_context_hash);

        let invalid_key_receipt = PolicyProofReceipt {
            entry: PolicyEntry {
                version: 1,
                key: "unknown.policy.key",
                value: 0xfeed,
            },
            policy_store_hash,
            receipt_event_seq: 7,
            receipt_event_hash: 0xabc,
            receipt_hash: 1,
        };
        assert_eq!(invalid_key_receipt.entry_hash(), None);
        assert_eq!(invalid_key_receipt.receipt_core_hash(), None);
        assert_eq!(invalid_key_receipt.verifier_context_hash(), None);
    }
}
