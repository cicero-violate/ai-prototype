//! Append-only policy store.

use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

use crate::capability::learning::{
    PolicyPromotion, POLICY_FEEDBACK_HASH, POLICY_PROMOTION_SOURCE_SEQ,
};

const POLICY_SCHEMA_VERSION: u64 = 1;
const POLICY_RECORD_ENTRY: u64 = 1;
const POLICY_KEY_PROMOTION_SOURCE_SEQ: u64 = 1;
const POLICY_KEY_FEEDBACK_HASH: u64 = 2;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PolicyEntry {
    pub version: u64,
    pub key: &'static str,
    pub value: u64,
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

fn mix(mut h: u64, x: u64) -> u64 {
    h ^= x;
    h = h.wrapping_mul(0x100000001b3);
    h
}
