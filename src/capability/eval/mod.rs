//! Eval capability.
//!
//! Eval is intentionally outside the kernel. The capability computes scores,
//! compares against policy thresholds, stores `EvalRecord`, then submits
//! `Evidence::EvalScore` as pass/fail evidence for `GateId::Eval`.

pub mod evolution;
pub mod record;

pub use self::evolution::{
    CandidateReceipt, CandidateReceiptInput, CandidateVerdict, EVOLUTION_LEDGER_RECORD,
    EVOLUTION_LEDGER_SCHEMA_VERSION, SelectionRecord, encode_candidate_receipt_ndjson,
};
pub use self::record::{
    EVAL_SCORECARD_RECORD, EVAL_SCORECARD_SCHEMA_VERSION, EvalDecision, EvalDimension, EvalRecord,
    EvalScorecardReceipt,
};
