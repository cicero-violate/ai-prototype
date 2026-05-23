//! Training artifact row schemas for the learning layer.
//!
//! Three append-only record families, all hash-linked:
//!
//! * `SymbolMutationRecord`       — which symbols were touched by a tool edit
//! * `ArchDecisionProposedRecord` / `ArchDecisionOutcomeRecord` — paired arch
//!   decision records joined by `decision_id = hash(def_path + ts + candidate_id)`
//! * `TaskSymbolIndexRecord`      — all symbols touched during one completed task

use serde::Serialize;

use crate::kernel::mix;

pub const SYMBOL_MUTATION_SCHEMA: &str = "canon.learning.symbol_mutation.v1";
pub const ARCH_DECISION_PROPOSED_SCHEMA: &str = "canon.learning.arch_decision_proposed.v1";
pub const ARCH_DECISION_OUTCOME_SCHEMA: &str = "canon.learning.arch_decision_outcome.v1";
pub const TASK_SYMBOL_INDEX_SCHEMA: &str = "canon.learning.task_symbol_index.v1";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum LearningArtifactKind {
    SymbolMutation = 1,
    ArchDecisionProposed = 2,
    ArchDecisionOutcome = 3,
    TaskSymbolIndex = 4,
}

// ── Symbol mutation ──────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize)]
pub struct SymbolMutationRecord {
    pub schema: &'static str,
    pub ts: u64,
    pub cycle: u64,
    pub task_node_hash: u64,
    pub task_title_hash: u64,
    pub file: String,
    pub def_paths: Vec<String>,
    pub edit_succeeded: bool,
    pub exit_status: u64,
    pub receipt_hash: u64,
    pub record_hash: u64,
}

impl SymbolMutationRecord {
    pub fn new(
        ts: u64,
        cycle: u64,
        task_node_hash: u64,
        task_title_hash: u64,
        file: String,
        def_paths: Vec<String>,
        edit_succeeded: bool,
        exit_status: u64,
        receipt_hash: u64,
    ) -> Self {
        let mut r = Self {
            schema: SYMBOL_MUTATION_SCHEMA,
            ts,
            cycle,
            task_node_hash,
            task_title_hash,
            file,
            def_paths,
            edit_succeeded,
            exit_status,
            receipt_hash,
            record_hash: 0,
        };
        r.record_hash = r.compute_hash();
        r
    }

    pub fn compute_hash(&self) -> u64 {
        let mut h = 0x534d_5554_4154_4e01u64;
        h = mix(h, self.ts);
        h = mix(h, self.cycle);
        h = mix(h, self.task_node_hash);
        h = mix(h, self.task_title_hash);
        h = mix(h, artifact_string_hash(&self.file));
        for dp in &self.def_paths {
            h = mix(h, artifact_string_hash(dp));
        }
        h = mix(h, self.edit_succeeded as u64);
        h = mix(h, self.exit_status);
        h = mix(h, self.receipt_hash);
        h.max(1)
    }

    pub fn is_valid(&self) -> bool {
        !self.file.is_empty() && self.record_hash != 0 && self.record_hash == self.compute_hash()
    }
}

// ── Refactor cost (embedded in arch decision proposed) ────────────────────────

#[derive(Clone, Debug, Serialize)]
pub struct RefactorCost {
    pub fan_in: u32,
    pub fan_out: u32,
    pub dependence_penalty: u32,
    pub api_churn_risk: f32,
    pub score: f32,
}

// ── Architectural decision: proposed ─────────────────────────────────────────

#[derive(Clone, Debug, Serialize)]
pub struct ArchDecisionProposedRecord {
    pub schema: &'static str,
    pub ts: u64,
    pub decision_id: u64,
    pub candidate_id: u64,
    pub def_path: String,
    pub symbol: String,
    pub from_layer: String,
    pub to_layer: String,
    pub confidence: f32,
    pub forbidden_words: Vec<String>,
    pub refactor_cost: RefactorCost,
    pub record_hash: u64,
}

impl ArchDecisionProposedRecord {
    pub fn new(
        ts: u64,
        candidate_id: u64,
        def_path: String,
        symbol: String,
        from_layer: String,
        to_layer: String,
        confidence: f32,
        forbidden_words: Vec<String>,
        refactor_cost: RefactorCost,
    ) -> Self {
        let did = decision_id(&def_path, ts, candidate_id);
        let mut r = Self {
            schema: ARCH_DECISION_PROPOSED_SCHEMA,
            ts,
            decision_id: did,
            candidate_id,
            def_path,
            symbol,
            from_layer,
            to_layer,
            confidence,
            forbidden_words,
            refactor_cost,
            record_hash: 0,
        };
        r.record_hash = r.compute_hash();
        r
    }

    pub fn compute_hash(&self) -> u64 {
        let mut h = 0x4152_4348_5052_4f01u64;
        h = mix(h, self.ts);
        h = mix(h, self.decision_id);
        h = mix(h, self.candidate_id);
        h = mix(h, artifact_string_hash(&self.def_path));
        h = mix(h, artifact_string_hash(&self.from_layer));
        h = mix(h, artifact_string_hash(&self.to_layer));
        h = mix(h, self.confidence.to_bits() as u64);
        h.max(1)
    }

    pub fn is_valid(&self) -> bool {
        self.decision_id != 0
            && !self.def_path.is_empty()
            && !self.from_layer.is_empty()
            && !self.to_layer.is_empty()
            && self.record_hash != 0
            && self.record_hash == self.compute_hash()
    }
}

/// `decision_id = hash(def_path + ts + candidate_id)` — joins proposed and outcome.
pub fn decision_id(def_path: &str, ts: u64, candidate_id: u64) -> u64 {
    let mut h = 0x4445_4349_4449_4401u64;
    h = mix(h, artifact_string_hash(def_path));
    h = mix(h, ts);
    h = mix(h, candidate_id);
    h.max(1)
}

// ── Architectural decision: outcome ──────────────────────────────────────────

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub enum ArchDecisionOutcome {
    Succeeded,
    Reverted,
    Failed,
}

impl ArchDecisionOutcome {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Succeeded => "succeeded",
            Self::Reverted => "reverted",
            Self::Failed => "failed",
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct ArchDecisionOutcomeRecord {
    pub schema: &'static str,
    pub ts: u64,
    pub decision_id: u64,
    pub outcome: String,
    pub outcome_receipt_hash: u64,
    pub record_hash: u64,
}

impl ArchDecisionOutcomeRecord {
    pub fn new(
        ts: u64,
        decision_id: u64,
        outcome: ArchDecisionOutcome,
        outcome_receipt_hash: u64,
    ) -> Self {
        let mut r = Self {
            schema: ARCH_DECISION_OUTCOME_SCHEMA,
            ts,
            decision_id,
            outcome: outcome.as_str().to_string(),
            outcome_receipt_hash,
            record_hash: 0,
        };
        r.record_hash = r.compute_hash();
        r
    }

    pub fn compute_hash(&self) -> u64 {
        let mut h = 0x4152_4348_4f55_5401u64;
        h = mix(h, self.ts);
        h = mix(h, self.decision_id);
        h = mix(h, artifact_string_hash(&self.outcome));
        h = mix(h, self.outcome_receipt_hash);
        h.max(1)
    }

    pub fn is_valid(&self) -> bool {
        self.decision_id != 0
            && !self.outcome.is_empty()
            && self.record_hash != 0
            && self.record_hash == self.compute_hash()
    }
}

// ── Task symbol index ─────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize)]
pub struct TouchedSymbol {
    pub def_path: String,
    pub layer: String,
    pub mutation_count: u32,
}

#[derive(Clone, Debug, Serialize)]
pub struct TaskSymbolIndexRecord {
    pub schema: &'static str,
    pub ts: u64,
    pub cycle: u64,
    pub task_node_hash: u64,
    pub task_title: String,
    pub task_status: String,
    pub symbols_touched: Vec<TouchedSymbol>,
    pub files_touched: Vec<String>,
    pub tool_sequence: Vec<String>,
    pub record_hash: u64,
}

impl TaskSymbolIndexRecord {
    pub fn new(
        ts: u64,
        cycle: u64,
        task_node_hash: u64,
        task_title: String,
        task_status: String,
        symbols_touched: Vec<TouchedSymbol>,
        files_touched: Vec<String>,
        tool_sequence: Vec<String>,
    ) -> Self {
        let mut r = Self {
            schema: TASK_SYMBOL_INDEX_SCHEMA,
            ts,
            cycle,
            task_node_hash,
            task_title,
            task_status,
            symbols_touched,
            files_touched,
            tool_sequence,
            record_hash: 0,
        };
        r.record_hash = r.compute_hash();
        r
    }

    pub fn compute_hash(&self) -> u64 {
        let mut h = 0x5441_534b_5359_4d01u64;
        h = mix(h, self.ts);
        h = mix(h, self.cycle);
        h = mix(h, self.task_node_hash);
        h = mix(h, artifact_string_hash(&self.task_title));
        h = mix(h, artifact_string_hash(&self.task_status));
        for sym in &self.symbols_touched {
            h = mix(h, artifact_string_hash(&sym.def_path));
            h = mix(h, sym.mutation_count as u64);
        }
        for file in &self.files_touched {
            h = mix(h, artifact_string_hash(file));
        }
        h.max(1)
    }

    pub fn is_valid(&self) -> bool {
        self.task_node_hash != 0
            && !self.task_title.is_empty()
            && self.record_hash != 0
            && self.record_hash == self.compute_hash()
    }
}

// ── Internal hash helper ──────────────────────────────────────────────────────

pub(crate) fn artifact_string_hash(value: &str) -> u64 {
    let mut h = 0x4c52_4e48_4153_4801u64;
    h = mix(h, value.len() as u64);
    for byte in value.as_bytes() {
        h = mix(h, *byte as u64);
    }
    h.max(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn symbol_mutation_record_hash_is_stable() {
        let r = SymbolMutationRecord::new(
            1_000_000,
            1,
            0xDEAD_BEEF,
            0xCAFE_BABE,
            "ai/src/lib.rs".to_string(),
            vec!["ai::lib::foo".to_string()],
            true,
            0,
            0xA11C_E001,
        );
        assert!(r.is_valid());
        assert_eq!(r.record_hash, r.compute_hash());
    }

    #[test]
    fn arch_decision_proposed_decision_id_is_stable() {
        let id1 = decision_id("ai::service::Foo", 1_000, 42);
        let id2 = decision_id("ai::service::Foo", 1_000, 42);
        assert_eq!(id1, id2);
        assert_ne!(id1, decision_id("ai::service::Bar", 1_000, 42));
    }

    #[test]
    fn arch_decision_outcome_record_is_valid() {
        let did = decision_id("ai::service::Foo", 1_000, 1);
        let r = ArchDecisionOutcomeRecord::new(2_000, did, ArchDecisionOutcome::Succeeded, 0xBEEF);
        assert!(r.is_valid());
        assert_eq!(r.outcome, "succeeded");
    }

    #[test]
    fn task_symbol_index_record_is_valid() {
        let r = TaskSymbolIndexRecord::new(
            3_000,
            5,
            0xABCD_1234,
            "fix hir deserialization".to_string(),
            "done".to_string(),
            vec![TouchedSymbol {
                def_path: "ai::codec::hir::HirRecord".to_string(),
                layer: "codec".to_string(),
                mutation_count: 3,
            }],
            vec!["ai/src/codec/hir.rs".to_string()],
            vec!["apply_patch".to_string(), "cargo_check".to_string()],
        );
        assert!(r.is_valid());
    }
}
