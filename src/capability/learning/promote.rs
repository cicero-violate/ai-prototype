//! Policy promotion payloads derived from TLog history.

use std::path::Path;

use crate::capability::{EvidenceProducer, EvidenceSubmission};
use crate::capability::policy::{
    PolicyEntry, PolicyStore, PolicyStoreError, POLICY_FEEDBACK_HASH,
    POLICY_PROMOTION_SOURCE_SEQ,
};
use crate::kernel::{mix, ControlEvent, EventKind, Evidence, GateId, Phase};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PolicyPromotion {
    pub source_seq: u64,
    pub promoted_policy_version: u64,
    pub judgment_seq: u64,
    pub eval_seq: u64,
    pub completion_seq: u64,
    pub promoted_policy_hash: u64,
    pub evidence: Evidence,
}

impl PolicyPromotion {
    pub fn from_tlog(tlog: &[ControlEvent], promoted_policy_version: u64) -> Option<Self> {
        let eval_event = tlog.iter().rev().find(|event| {
            event.kind == EventKind::Advanced
                && event.to == Phase::Persist
                && event.evidence == Evidence::EvalScore
        })?;

        let judgment_event = tlog.iter().rev().find(|event| {
            event.seq < eval_event.seq
                && event.kind == EventKind::Advanced
                && event.evidence == Evidence::JudgmentRecord
        })?;

        let completion_event = tlog
            .iter()
            .rev()
            .find(|event| event.kind == EventKind::Learned || event.kind == EventKind::Completed)?;

        let promoted_policy_hash = promotion_hash(
            promoted_policy_version,
            judgment_event,
            eval_event,
            completion_event,
        );

        Some(Self {
            source_seq: eval_event.seq,
            promoted_policy_version,
            judgment_seq: judgment_event.seq,
            eval_seq: eval_event.seq,
            completion_seq: completion_event.seq,
            promoted_policy_hash,
            evidence: Evidence::PolicyPromotion,
        })
    }

    pub fn is_valid(&self) -> bool {
        self.source_seq != 0
            && self.promoted_policy_version != 0
            && self.judgment_seq != 0
            && self.eval_seq == self.source_seq
            && self.completion_seq >= self.eval_seq
            && self.promoted_policy_hash != 0
            && self.evidence == Evidence::PolicyPromotion
    }

    pub fn submission(&self) -> EvidenceSubmission {
        EvidenceSubmission::with_payload(
            GateId::Learning,
            Evidence::PolicyPromotion,
            self.is_valid(),
            self.promoted_policy_hash,
        )
    }
}

impl EvidenceProducer for PolicyPromotion {
    type Record = PolicyPromotion;

    fn record(&self) -> &Self::Record {
        self
    }

    fn submission(&self) -> EvidenceSubmission {
        PolicyPromotion::submission(self)
    }
}

impl PolicyStore {
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

        self.entries()
            .last()
            .ok_or(PolicyStoreError::InvalidPromotion)
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

        self.entries()
            .last()
            .ok_or(PolicyStoreError::InvalidPromotion)
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
}

fn promotion_hash(
    promoted_policy_version: u64,
    judgment_event: &ControlEvent,
    eval_event: &ControlEvent,
    completion_event: &ControlEvent,
) -> u64 {
    let mut h = 0x732f_6a61_2d70_6f6cu64;
    h = mix(h, promoted_policy_version);
    h = mix(h, judgment_event.seq);
    h = mix(h, judgment_event.self_hash);
    h = mix(h, eval_event.seq);
    h = mix(h, eval_event.self_hash);
    h = mix(h, completion_event.seq);
    h = mix(h, completion_event.self_hash);
    h.max(1)
}
