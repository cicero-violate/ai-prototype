//! Typed LLM task context and turn receipt.
//!
//! Target shape:
//!   LlmTaskContext -> capability::run_turn -> LlmTurnRecord -> LlmTurnReceipt
//!                  -> EvidenceSubmission -> command ingress -> TLog

use crate::capability::EvidenceSubmission;
use crate::kernel::{mix, Evidence, GateId};

const LLM_TASK_RECEIPT_SCHEMA_VERSION: u64 = 1;
const LLM_TASK_RECEIPT_SEED: u64 = 0x4c4c_4d54_4153_4b52; // "LLMTASKR"

/// Task ownership context for an LLM capability run.
///
/// Captures the durable lease identity (plan node, worker, claim_id) plus the
/// model that will be called.  Every `LlmTurnRecord` produced during this task
/// must carry this context's hash so receipts can be traced back to their task.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LlmTaskContext {
    pub schema_version: u64,
    /// FNV hash of the plan node id.
    pub node_id_hash: u64,
    /// FNV hash of the worker id.
    pub worker_id_hash: u64,
    /// Claim id from the task lease (ownership proof).
    pub claim_id: u64,
    /// Hash of the model / provider being invoked.
    pub model_hash: u64,
    /// Client-chosen idempotency key for the assignment.
    pub idempotency_key: u64,
}

impl LlmTaskContext {
    pub fn new(
        node_id: &str,
        worker_id: &str,
        claim_id: u64,
        model_hash: u64,
        idempotency_key: u64,
    ) -> Option<Self> {
        if node_id.trim().is_empty() || worker_id.trim().is_empty() {
            return None;
        }
        let node_id_hash = hash_text(node_id);
        let worker_id_hash = hash_text(worker_id);
        if node_id_hash == 0 || worker_id_hash == 0 || claim_id == 0 || model_hash == 0 {
            return None;
        }
        Some(Self {
            schema_version: LLM_TASK_RECEIPT_SCHEMA_VERSION,
            node_id_hash,
            worker_id_hash,
            claim_id,
            model_hash,
            idempotency_key,
        })
    }

    pub fn is_valid(&self) -> bool {
        self.schema_version == LLM_TASK_RECEIPT_SCHEMA_VERSION
            && self.node_id_hash != 0
            && self.worker_id_hash != 0
            && self.claim_id != 0
            && self.model_hash != 0
    }

    /// Deterministic hash of this context, used as a foreign key in turn records.
    pub fn context_hash(&self) -> u64 {
        [
            self.schema_version,
            self.node_id_hash,
            self.worker_id_hash,
            self.claim_id,
            self.model_hash,
            self.idempotency_key,
        ]
        .iter()
        .fold(LLM_TASK_RECEIPT_SEED, |acc, &v| mix(acc, v))
        .max(1)
    }
}

/// A single LLM turn: prompt in, response out, token cost.
///
/// One task may consist of many turns (multi-turn agent loop).  Each turn
/// carries `task_context_hash` so replay can associate turns with their task.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LlmTurnRecord {
    pub schema_version: u64,
    /// `LlmTaskContext::context_hash()` for this task.
    pub task_context_hash: u64,
    /// Monotonically increasing turn number within the task (1-based).
    pub turn_seq: u32,
    /// Hash of the full prompt submitted to the model.
    pub prompt_hash: u64,
    /// Hash of the model's response.
    pub response_hash: u64,
    /// Tokens consumed by this turn (prompt + completion).
    pub token_count: u32,
    /// Model / provider identity hash.
    pub model_hash: u64,
    /// `true` if the turn produced parseable structured output.
    pub structured: bool,
}

impl LlmTurnRecord {
    pub fn new(
        context: &LlmTaskContext,
        turn_seq: u32,
        prompt_hash: u64,
        response_hash: u64,
        token_count: u32,
        structured: bool,
    ) -> Option<Self> {
        if !context.is_valid()
            || turn_seq == 0
            || prompt_hash == 0
            || response_hash == 0
            || token_count == 0
        {
            return None;
        }
        Some(Self {
            schema_version: LLM_TASK_RECEIPT_SCHEMA_VERSION,
            task_context_hash: context.context_hash(),
            turn_seq,
            prompt_hash,
            response_hash,
            token_count,
            model_hash: context.model_hash,
            structured,
        })
    }

    pub fn is_valid(&self) -> bool {
        self.schema_version == LLM_TASK_RECEIPT_SCHEMA_VERSION
            && self.task_context_hash != 0
            && self.turn_seq != 0
            && self.prompt_hash != 0
            && self.response_hash != 0
            && self.token_count != 0
            && self.model_hash != 0
    }

    /// Deterministic self-hash, used as the foreign key in `LlmTurnReceipt`.
    pub fn record_hash(&self) -> u64 {
        [
            self.schema_version,
            self.task_context_hash,
            self.turn_seq as u64,
            self.prompt_hash,
            self.response_hash,
            self.token_count as u64,
            self.model_hash,
            u64::from(self.structured),
        ]
        .iter()
        .fold(LLM_TASK_RECEIPT_SEED, |acc, &v| mix(acc, v))
        .max(1)
    }
}

/// Typed receipt for a completed LLM turn.
///
/// Binds `LlmTurnRecord` identity to the TLog event that accepted its evidence.
/// A valid `LlmTurnReceipt` is required before the worker may complete or learn
/// from this turn.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LlmTurnReceipt {
    pub schema_version: u64,
    /// `LlmTurnRecord::record_hash()` this receipt covers.
    pub turn_record_hash: u64,
    /// TLog event seq that committed the turn's evidence.
    pub event_seq: u64,
    /// TLog event hash (replay anchor).
    pub event_hash: u64,
    /// Task lease claim_id (durable ownership proof).
    pub claim_id: u64,
    /// `true` if the receipt is bound to a TLog event.
    pub replay_bound: bool,
    /// Self-validating hash; must equal `expected_receipt_hash()`.
    pub receipt_hash: u64,
}

impl LlmTurnReceipt {
    /// Build a receipt from a completed turn after its evidence lands in the TLog.
    pub fn from_turn_record(
        record: &LlmTurnRecord,
        event_seq: u64,
        event_hash: u64,
        claim_id: u64,
    ) -> Option<Self> {
        if !record.is_valid() || event_seq == 0 || event_hash == 0 || claim_id == 0 {
            return None;
        }
        let mut r = Self {
            schema_version: LLM_TASK_RECEIPT_SCHEMA_VERSION,
            turn_record_hash: record.record_hash(),
            event_seq,
            event_hash,
            claim_id,
            replay_bound: true,
            receipt_hash: 0,
        };
        r.receipt_hash = r.expected_receipt_hash();
        Some(r)
    }

    pub fn expected_receipt_hash(&self) -> u64 {
        [
            self.schema_version,
            self.turn_record_hash,
            self.event_seq,
            self.event_hash,
            self.claim_id,
            u64::from(self.replay_bound),
        ]
        .iter()
        .fold(LLM_TASK_RECEIPT_SEED, |acc, &v| mix(acc, v))
        .max(1)
    }

    pub fn is_valid(&self) -> bool {
        self.schema_version == LLM_TASK_RECEIPT_SCHEMA_VERSION
            && self.turn_record_hash != 0
            && self.event_seq != 0
            && self.event_hash != 0
            && self.claim_id != 0
            && self.receipt_hash != 0
            && self.receipt_hash == self.expected_receipt_hash()
    }

    /// `EvidenceSubmission` for routing this receipt through command ingress.
    pub fn submission(&self) -> EvidenceSubmission {
        EvidenceSubmission::with_payload(
            GateId::Execution,
            Evidence::ExecutionReceipt,
            self.is_valid(),
            self.receipt_hash,
        )
    }
}

fn hash_text(value: &str) -> u64 {
    value
        .as_bytes()
        .iter()
        .fold(0xcbf2_9ce4_8422_2325u64, |h, &b| {
            h.wrapping_mul(0x0000_0100_0000_01b3) ^ u64::from(b)
        })
        .max(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn context() -> LlmTaskContext {
        LlmTaskContext::new("node-1", "worker-a", 42, 0xdead_beef, 7)
            .expect("test setup should succeed")
    }

    #[test]
    fn llm_task_context_is_valid_with_all_fields() {
        let ctx = context();
        assert!(ctx.is_valid());
        assert_ne!(ctx.context_hash(), 0);
    }

    #[test]
    fn llm_task_context_rejects_empty_node_id() {
        assert!(LlmTaskContext::new("", "worker-a", 1, 0xabc, 0).is_none());
    }

    #[test]
    fn llm_task_context_rejects_zero_claim_id() {
        assert!(LlmTaskContext::new("node-1", "worker-a", 0, 0xabc, 0).is_none());
    }

    #[test]
    fn llm_turn_record_is_valid_with_all_fields() {
        let ctx = context();
        let record = LlmTurnRecord::new(&ctx, 1, 0xaaaa, 0xbbbb, 100, true)
            .expect("test setup should succeed");
        assert!(record.is_valid());
        assert_ne!(record.record_hash(), 0);
        assert_eq!(record.task_context_hash, ctx.context_hash());
    }

    #[test]
    fn llm_turn_record_rejects_zero_turn_seq() {
        let ctx = context();
        assert!(LlmTurnRecord::new(&ctx, 0, 0xaaaa, 0xbbbb, 100, true).is_none());
    }

    #[test]
    fn llm_turn_record_rejects_zero_token_count() {
        let ctx = context();
        assert!(LlmTurnRecord::new(&ctx, 1, 0xaaaa, 0xbbbb, 0, true).is_none());
    }

    #[test]
    fn llm_turn_receipt_is_valid_and_self_consistent() {
        let ctx = context();
        let record = LlmTurnRecord::new(&ctx, 1, 0xaaaa, 0xbbbb, 100, true)
            .expect("test setup should succeed");
        let receipt = LlmTurnReceipt::from_turn_record(&record, 3, 0xcccc_dddd, 42)
            .expect("test setup should succeed");

        assert!(receipt.is_valid());
        assert_eq!(receipt.turn_record_hash, record.record_hash());
        assert_eq!(receipt.receipt_hash, receipt.expected_receipt_hash());
        assert!(receipt.replay_bound);
    }

    #[test]
    fn llm_turn_receipt_rejects_zero_event_seq() {
        let ctx = context();
        let record = LlmTurnRecord::new(&ctx, 1, 0xaaaa, 0xbbbb, 100, true)
            .expect("test setup should succeed");
        assert!(LlmTurnReceipt::from_turn_record(&record, 0, 0xcccc, 42).is_none());
    }

    #[test]
    fn llm_turn_receipt_submission_carries_execution_receipt_evidence() {
        let ctx = context();
        let record = LlmTurnRecord::new(&ctx, 1, 0xaaaa, 0xbbbb, 100, true)
            .expect("test setup should succeed");
        let receipt = LlmTurnReceipt::from_turn_record(&record, 3, 0xcccc_dddd, 42)
            .expect("test setup should succeed");

        let sub = receipt.submission();
        assert_eq!(sub.gate, GateId::Execution);
        assert_eq!(sub.evidence, Evidence::ExecutionReceipt);
        assert!(sub.passed);
    }

    #[test]
    fn tampered_receipt_hash_invalidates_receipt() {
        let ctx = context();
        let record = LlmTurnRecord::new(&ctx, 1, 0xaaaa, 0xbbbb, 100, true)
            .expect("test setup should succeed");
        let mut receipt = LlmTurnReceipt::from_turn_record(&record, 3, 0xcccc_dddd, 42)
            .expect("test setup should succeed");

        receipt.receipt_hash ^= 1;
        assert!(!receipt.is_valid());
    }

    #[test]
    fn context_hash_changes_when_node_id_changes() {
        let a = LlmTaskContext::new("node-a", "w", 1, 0xabc, 0).expect("test setup should succeed");
        let b = LlmTaskContext::new("node-b", "w", 1, 0xabc, 0).expect("test setup should succeed");
        assert_ne!(a.context_hash(), b.context_hash());
    }
}
