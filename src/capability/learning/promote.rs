//! Policy promotion and distillation payloads derived from TLog history.

use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::capability::policy::{
    PolicyEntry, PolicyStore, PolicyStoreError, POLICY_FEEDBACK_HASH, POLICY_PROMOTION_SOURCE_SEQ,
};
use crate::capability::{EvidenceProducer, EvidenceSubmission};
use crate::kernel::{mix, ControlEvent, EventKind, Evidence, GateId, Phase};

pub const DISTILLATION_ROW_SCHEMA_VERSION: u64 = 1;
pub const DISTILLATION_ROW_RECORD: u64 = 0x4449_5354_3031_3031;

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

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DistillationExportError {
    InvalidTlog,
    NoVerifiedPromotion,
    InvalidRow,
    Io,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DistillationExportInput {
    pub promoted_policy_version: u64,
    pub instruction_hash: u64,
    pub input_state_hash: u64,
    pub action_hash: u64,
    pub output_hash: u64,
    pub score: u64,
    pub proof_hash: u64,
    pub minimum_score: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DistillationExportReceipt {
    pub schema_version: u64,
    pub source_event: u64,
    pub row_count: u64,
    pub output_hash: u64,
    pub proof_hash: u64,
    pub receipt_hash: u64,
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

    pub fn encode_ndjson(&self) -> Option<String> {
        self.expected_row_hash().and_then(|expected| {
            (expected == self.row_hash).then(|| {
                format!(
                    "[{},{},{},{},{},{},{},{},{},{}]\n",
                    self.schema_version,
                    DISTILLATION_ROW_RECORD,
                    self.instruction_hash,
                    self.input_state_hash,
                    self.action_hash,
                    self.output_hash,
                    self.score,
                    self.proof_hash,
                    self.source_event,
                    self.row_hash,
                )
            })
        })
    }
}

impl DistillationExportReceipt {
    pub fn is_valid_for(&self, row: &DistillationRow) -> bool {
        self.schema_version == DISTILLATION_ROW_SCHEMA_VERSION
            && self.source_event == row.source_event
            && self.row_count == 1
            && self.output_hash == row.row_hash
            && self.proof_hash == row.proof_hash
            && self.receipt_hash == self.expected_receipt_hash().unwrap_or(0)
    }

    pub fn expected_receipt_hash(&self) -> Option<u64> {
        if self.schema_version != DISTILLATION_ROW_SCHEMA_VERSION
            || self.source_event == 0
            || self.row_count == 0
            || self.output_hash == 0
            || self.proof_hash == 0
        {
            return None;
        }
        let mut h = 0x4453_544c_584f_3031u64;
        h = mix(h, self.schema_version);
        h = mix(h, self.source_event);
        h = mix(h, self.row_count);
        h = mix(h, self.output_hash);
        h = mix(h, self.proof_hash);
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PolicyPromotionEntryKind {
    SourceSeq,
    FeedbackHash,
}

fn policy_promotion_entry(
    promotion: &PolicyPromotion,
    kind: PolicyPromotionEntryKind,
) -> Result<PolicyEntry, PolicyStoreError> {
    if !promotion.is_valid() {
        return Err(PolicyStoreError::InvalidPromotion);
    }

    let (key, value) = match kind {
        PolicyPromotionEntryKind::SourceSeq => (POLICY_PROMOTION_SOURCE_SEQ, promotion.source_seq),
        PolicyPromotionEntryKind::FeedbackHash => {
            (POLICY_FEEDBACK_HASH, promotion.promoted_policy_hash)
        }
    };

    Ok(PolicyEntry {
        version: promotion.promoted_policy_version,
        key,
        value,
    })
}

impl PolicyStore {
    fn append_valid_promotion_entry(
        &mut self,
        promotion: PolicyPromotion,
        kind: PolicyPromotionEntryKind,
    ) -> Result<&PolicyEntry, PolicyStoreError> {
        let entry = policy_promotion_entry(&promotion, kind)?;
        self.try_append(entry)?;
        self.entries()
            .last()
            .ok_or(PolicyStoreError::InvalidPromotion)
    }

    fn append_in_memory_promotion_route(
        &mut self,
        promotion: PolicyPromotion,
        kind: PolicyPromotionEntryKind,
    ) -> Result<&PolicyEntry, PolicyStoreError> {
        self.append_valid_promotion_entry(promotion, kind)
    }

    fn append_valid_promotion_entry_durable(
        &mut self,
        path: impl AsRef<Path>,
        promotion: PolicyPromotion,
        kind: PolicyPromotionEntryKind,
    ) -> Result<&PolicyEntry, PolicyStoreError> {
        let entry = policy_promotion_entry(&promotion, kind)?;
        self.append_durable(path, entry)
    }

    pub fn promote(
        &mut self,
        promotion: PolicyPromotion,
    ) -> Result<&PolicyEntry, PolicyStoreError> {
        self.append_in_memory_promotion_route(promotion, PolicyPromotionEntryKind::SourceSeq)
    }

    pub fn promote_feedback(
        &mut self,
        promotion: PolicyPromotion,
    ) -> Result<&PolicyEntry, PolicyStoreError> {
        self.append_in_memory_promotion_route(promotion, PolicyPromotionEntryKind::FeedbackHash)
    }

    pub fn promote_durable(
        &mut self,
        path: impl AsRef<Path>,
        promotion: PolicyPromotion,
    ) -> Result<&PolicyEntry, PolicyStoreError> {
        self.append_valid_promotion_entry_durable(
            path,
            promotion,
            PolicyPromotionEntryKind::SourceSeq,
        )
    }

    pub fn promote_feedback_durable(
        &mut self,
        path: impl AsRef<Path>,
        promotion: PolicyPromotion,
    ) -> Result<&PolicyEntry, PolicyStoreError> {
        self.append_valid_promotion_entry_durable(
            path,
            promotion,
            PolicyPromotionEntryKind::FeedbackHash,
        )
    }
}

pub fn export_verified_distillation_row(
    tlog: &[ControlEvent],
    path: impl AsRef<Path>,
    input: DistillationExportInput,
) -> Result<DistillationExportReceipt, DistillationExportError> {
    verify_distillation_export_input(&input)?;
    crate::runtime::verify_tlog(tlog).map_err(|_| DistillationExportError::InvalidTlog)?;

    let promotion = PolicyPromotion::from_tlog(tlog, input.promoted_policy_version)
        .ok_or(DistillationExportError::NoVerifiedPromotion)?;
    if input.score < input.minimum_score || input.proof_hash != promotion.promoted_policy_hash {
        return Err(DistillationExportError::NoVerifiedPromotion);
    }
    let eval_event = tlog
        .iter()
        .find(|event| event.seq == promotion.eval_seq)
        .ok_or(DistillationExportError::NoVerifiedPromotion)?;
    if eval_event.evidence != Evidence::EvalScore
        || eval_event.to != Phase::Persist
        || eval_event.state_after.gates.eval.status != crate::kernel::GateStatus::Pass
        || !eval_event.state_after.packet.objective_complete()
        || !eval_event.state_after.packet.lineage_valid()
    {
        return Err(DistillationExportError::NoVerifiedPromotion);
    }

    let row = DistillationRow::from_policy_promotion(
        &promotion,
        input.instruction_hash,
        input.input_state_hash,
        input.action_hash,
        input.output_hash,
        input.score,
        input.proof_hash,
    )
    .ok_or(DistillationExportError::InvalidRow)?;
    let encoded = row
        .encode_ndjson()
        .ok_or(DistillationExportError::InvalidRow)?;
    write_distillation_output(path.as_ref(), encoded.as_bytes())?;

    let mut receipt = DistillationExportReceipt {
        schema_version: DISTILLATION_ROW_SCHEMA_VERSION,
        source_event: row.source_event,
        row_count: 1,
        output_hash: row.row_hash,
        proof_hash: row.proof_hash,
        receipt_hash: 0,
    };
    receipt.receipt_hash = receipt
        .expected_receipt_hash()
        .ok_or(DistillationExportError::InvalidRow)?;
    receipt
        .is_valid_for(&row)
        .then_some(receipt)
        .ok_or(DistillationExportError::InvalidRow)
}

fn verify_distillation_export_input(
    input: &DistillationExportInput,
) -> Result<(), DistillationExportError> {
    if input.promoted_policy_version == 0
        || input.instruction_hash == 0
        || input.input_state_hash == 0
        || input.action_hash == 0
        || input.output_hash == 0
        || input.score == 0
        || input.proof_hash == 0
        || input.minimum_score == 0
    {
        return Err(DistillationExportError::InvalidRow);
    }
    Ok(())
}

fn write_distillation_output(path: &Path, bytes: &[u8]) -> Result<(), DistillationExportError> {
    let tmp_path = temporary_distillation_path(path);
    {
        let mut file = File::create(&tmp_path).map_err(|_| DistillationExportError::Io)?;
        file.write_all(bytes)
            .map_err(|_| DistillationExportError::Io)?;
        file.sync_all().map_err(|_| DistillationExportError::Io)?;
    }
    fs::rename(&tmp_path, path).map_err(|_| DistillationExportError::Io)?;
    sync_distillation_parent_dir(path)
}

fn temporary_distillation_path(path: &Path) -> PathBuf {
    let mut tmp = path.to_path_buf();
    let suffix = match path.extension().and_then(|v| v.to_str()) {
        Some(ext) if !ext.is_empty() => format!("{ext}.tmp"),
        _ => "tmp".to_string(),
    };
    tmp.set_extension(suffix);
    tmp
}

fn sync_distillation_parent_dir(path: &Path) -> Result<(), DistillationExportError> {
    let Some(parent) = path.parent() else {
        return Ok(());
    };
    if parent.as_os_str().is_empty() {
        return Ok(());
    }
    let dir = File::open(parent).map_err(|_| DistillationExportError::Io)?;
    dir.sync_all().map_err(|_| DistillationExportError::Io)
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
