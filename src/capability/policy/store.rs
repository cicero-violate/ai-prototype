//! Append-only policy store.

use crate::capability::learning::{PolicyPromotion, POLICY_PROMOTION_SOURCE_SEQ};

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
        self.entries.push(entry);
        Ok(())
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

    pub fn entries(&self) -> &[PolicyEntry] {
        &self.entries
    }
}
