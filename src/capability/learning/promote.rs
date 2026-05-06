//! Policy promotion and distillation payloads derived from TLog history.

use std::path::Path;

use crate::capability::{EvidenceProducer, EvidenceSubmission};
use crate::capability::policy::{
    PolicyEntry, PolicyStore, PolicyStoreError, POLICY_FEEDBACK_HASH,
    POLICY_PROMOTION_SOURCE_SEQ,
};
use crate::kernel::{mix, ControlEvent, EventKind, Evidence, GateId, Phase};

pub const DISTILLATION_ROW_SCHEMA_VERSION: u64 = 1;

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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DistillationRow {
    pub schema_version: u64,
    pub instruction_hash: u64,
    pub input_state_hash: u64,
    pub action_hash: u64,
    pub output_hash: u64,
    pub score: u64,
    pub proof_hash: u64,
    pub source_event: u64,
    pub row_hash: u64,
}

impl DistillationRow {
    pub fn from_policy_promotion(
        promotion: &PolicyPromotion,
        instruction_hash: u64,
        input_state_hash: u64,
        action_hash: u64,
        output_hash: u64,
        score: u64,
        proof_hash: u64,
    ) -> Option<Self> {
        if !promotion.is_valid()
            || instruction_hash == 0
            || input_state_hash == 0
            || action_hash == 0
            || output_hash == 0
            || score == 0
            || proof_hash != promotion.promoted_policy_hash
        {
            return None;
        }

        let mut row = Self {
            schema_version: DISTILLATION_ROW_SCHEMA_VERSION,
            instruction_hash,
            input_state_hash,
            action_hash,
            output_hash,
            score,
            proof_hash,
            source_event: promotion.source_seq,
            row_hash: 0,
        };
        row.row_hash = row.expected_row_hash()?;
        row.is_valid_for(promotion).then_some(row)
    }

    pub fn is_valid_for(&self, promotion: &PolicyPromotion) -> bool {
        promotion.is_valid()
            && self.schema_version == DISTILLATION_ROW_SCHEMA_VERSION
            && self.instruction_hash != 0
            && self.input_state_hash != 0
            && self.action_hash != 0
            && self.output_hash != 0
            && self.score != 0
            && self.proof_hash == promotion.promoted_policy_hash
            && self.source_event == promotion.source_seq
            && self.row_hash != 0
            && self.row_hash == self.expected_row_hash().unwrap_or(0)
    }

    pub fn expected_row_hash(&self) -> Option<u64> {
        if self.schema_version != DISTILLATION_ROW_SCHEMA_VERSION
            || self.instruction_hash == 0
            || self.input_state_hash == 0
            || self.action_hash == 0
            || self.output_hash == 0
            || self.score == 0
            || self.proof_hash == 0
            || self.source_event == 0
        {
            return None;
        }

        let mut h = 0x4449_5354_494c_4c31u64;
        h = mix(h, self.schema_version);
        h = mix(h, self.instruction_hash);
        h = mix(h, self.input_state_hash);
        h = mix(h, self.action_hash);
        h = mix(h, self.output_hash);
        h = mix(h, self.score);
        h = mix(h, self.proof_hash);
        h = mix(h, self.source_event);
        Some(h.max(1))
    }
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
