//! Judgment payload owned outside the kernel.

use crate::capability::context::ContextRecord;
use crate::capability::policy::{PolicyLookupReceipt, PolicyStore, POLICY_FEEDBACK_HASH};
use crate::capability::{EvidenceProducer, EvidenceSubmission};
use crate::kernel::{mix, Evidence, GateId};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JudgmentRecord {
    pub decision_id: u64,
    pub policy_version: u64,
    pub rationale_hash: u64,
}

impl JudgmentRecord {
    pub fn is_valid(&self) -> bool {
        self.decision_id != 0 && self.policy_version != 0 && self.rationale_hash != 0
    }

    pub fn submission(&self) -> EvidenceSubmission {
        EvidenceSubmission::with_payload(
            GateId::Judgment,
            Evidence::JudgmentRecord,
            self.is_valid(),
            judgment_payload_hash(self),
        )
    }
}

impl EvidenceProducer for JudgmentRecord {
    type Record = JudgmentRecord;

    fn record(&self) -> &Self::Record {
        self
    }

    fn submission(&self) -> EvidenceSubmission {
        JudgmentRecord::submission(self)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PolicyJudgmentDecision {
    PolicyHit,
    PolicyMiss,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PolicyJudgmentRecord {
    pub context_hash: u64,
    pub policy_version: u64,
    pub policy_hash: u64,
    pub policy_feedback_hash: u64,
    pub policy_lookup_receipt_hash: u64,
    pub decision_id: u64,
    pub rationale_hash: u64,
    pub record_hash: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PolicyReuseReceipt {
    pub schema_version: u64,
    pub record_type: &'static str,
    pub retained_record_count: usize,
    pub policy_hit_count: usize,
    pub policy_miss_count: usize,
    pub avoided_llm_call_count: usize,
    pub hit_rate_bps: u64,
    pub record_set_hash: u64,
    pub verdict: &'static str,
    pub receipt_hash: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PolicyReuseTrendReceipt {
    pub schema_version: u64,
    pub record_type: &'static str,
    pub baseline_receipt_hash: u64,
    pub current_receipt_hash: u64,
    pub baseline_retained_record_count: usize,
    pub current_retained_record_count: usize,
    pub baseline_hit_rate_bps: u64,
    pub current_hit_rate_bps: u64,
    pub hit_rate_delta_bps: i64,
    pub avoided_llm_call_delta: i64,
    pub trend_status: &'static str,
    pub verdict: &'static str,
    pub receipt_hash: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PolicyReuseLedgerSummaryReceipt {
    pub schema_version: u64,
    pub record_type: &'static str,
    pub policy_hits: usize,
    pub policy_misses: usize,
    pub llm_fallbacks: usize,
    pub validation_passes: usize,
    pub validation_failures: usize,
    pub reuse_rate_bps: u64,
    pub regression_flag: bool,
    pub source_receipt_hash: u64,
    pub receipt_hash: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PolicyReuseScaleTraceReceipt {
    pub schema_version: u64,
    pub record_type: &'static str,
    pub batch_size: usize,
    pub policy_hits: usize,
    pub policy_misses: usize,
    pub llm_fallbacks: usize,
    pub validation_passes: usize,
    pub validation_failures: usize,
    pub reuse_rate_bps: u64,
    pub regression_flag: bool,
    pub avoided_llm_calls_per_batch: usize,
    pub source_receipt_hash: u64,
    pub receipt_hash: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PolicyReusePerformanceCostTrendReceipt {
    pub schema_version: u64,
    pub record_type: &'static str,
    pub batch_size: usize,
    pub avoided_llm_calls_per_batch: usize,
    pub reuse_rate_bps: u64,
    pub validation_expected_count_guarded_tests: usize,
    pub estimated_ms_per_guarded_test: u64,
    pub runtime_budget_status: &'static str,
    pub validation_cost_verdict: &'static str,
    pub cost_regression_flag: bool,
    pub source_scale_trace_hash: u64,
    pub source_validation_duration_hash: u64,
    pub source_runtime_performance_hash: u64,
    pub receipt_hash: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PolicyReuseCostCatalogReceipt {
    pub schema_version: u64,
    pub record_type: &'static str,
    pub catalog_version: u64,
    pub evidence_family_count: usize,
    pub healthy_mode_count: usize,
    pub regression_mode_count: usize,
    pub retained_fixture_count: usize,
    pub required_healthy_modes_present: bool,
    pub required_regression_modes_present: bool,
    pub summary_complete: bool,
    pub missing_required_modes: &'static str,
    pub source_policy_reuse_hash: u64,
    pub source_scale_trace_hash: u64,
    pub source_performance_cost_trend_hash: u64,
    pub source_validation_health_hash: u64,
    pub source_validation_duration_hash: u64,
    pub source_runtime_performance_hash: u64,
    pub catalog_hash: u64,
    pub receipt_hash: u64,
}

impl PolicyReuseLedgerSummaryReceipt {
    pub fn from_reuse_validation_counts(
        reuse: &PolicyReuseReceipt,
        validation_passes: usize,
        validation_failures: usize,
    ) -> Self {
        let llm_fallbacks = reuse.policy_miss_count;
        let regression_flag = validation_failures > 0 || !reuse.passed();
        let mut receipt = Self {
            schema_version: 1,
            record_type: "policy_reuse_ledger_summary",
            policy_hits: reuse.policy_hit_count,
            policy_misses: reuse.policy_miss_count,
            llm_fallbacks,
            validation_passes,
            validation_failures,
            reuse_rate_bps: reuse.hit_rate_bps,
            regression_flag,
            source_receipt_hash: reuse.receipt_hash,
            receipt_hash: 0,
        };
        receipt.receipt_hash = policy_reuse_ledger_summary_receipt_hash(&receipt);
        receipt
    }

    pub fn empty() -> Self {
        let mut receipt = Self {
            schema_version: 1,
            record_type: "policy_reuse_ledger_summary",
            policy_hits: 0,
            policy_misses: 0,
            llm_fallbacks: 0,
            validation_passes: 0,
            validation_failures: 0,
            reuse_rate_bps: 0,
            regression_flag: false,
            source_receipt_hash: policy_reuse_empty_source_hash(),
            receipt_hash: 0,
        };
        receipt.receipt_hash = policy_reuse_ledger_summary_receipt_hash(&receipt);
        receipt
    }

    pub fn to_json(&self) -> String {
        format!(
            "{{\"schema_version\":{},\"record_type\":\"{}\",\"policy_hits\":{},\"policy_misses\":{},\"llm_fallbacks\":{},\"validation_passes\":{},\"validation_failures\":{},\"reuse_rate_bps\":{},\"regression_flag\":{},\"source_receipt_hash\":{},\"receipt_hash\":{}}}",
            self.schema_version,
            self.record_type,
            self.policy_hits,
            self.policy_misses,
            self.llm_fallbacks,
            self.validation_passes,
            self.validation_failures,
            self.reuse_rate_bps,
            self.regression_flag,
            self.source_receipt_hash,
            self.receipt_hash,
        )
    }

    pub fn is_valid(&self) -> bool {
        self.schema_version == 1
            && matches!(
                self.record_type,
                "policy_reuse_ledger_summary"
                    | "policy_reuse_ledger_summary_smoke"
                    | "policy_reuse_ledger_summary_regression_smoke"
            )
            && self.llm_fallbacks == self.policy_misses
            && self.reuse_rate_bps <= 10_000
            && self.source_receipt_hash != 0
            && self.receipt_hash != 0
            && self.receipt_hash == policy_reuse_ledger_summary_receipt_hash(self)
    }

    pub fn passed(&self) -> bool {
        self.is_valid() && !self.regression_flag
    }
}

impl PolicyReuseScaleTraceReceipt {
    pub fn from_reuse_validation_counts(
        reuse: &PolicyReuseReceipt,
        batch_size: usize,
        validation_passes: usize,
        validation_failures: usize,
    ) -> Self {
        let llm_fallbacks = reuse.policy_miss_count;
        let regression_flag = validation_failures > 0 || !reuse.passed();
        let avoided_llm_calls_per_batch = reuse.policy_hit_count;
        let mut receipt = Self {
            schema_version: 1,
            record_type: "policy_reuse_scale_trace",
            batch_size,
            policy_hits: reuse.policy_hit_count,
            policy_misses: reuse.policy_miss_count,
            llm_fallbacks,
            validation_passes,
            validation_failures,
            reuse_rate_bps: reuse.hit_rate_bps,
            regression_flag,
            avoided_llm_calls_per_batch,
            source_receipt_hash: reuse.receipt_hash,
            receipt_hash: 0,
        };
        receipt.receipt_hash = policy_reuse_scale_trace_receipt_hash(&receipt);
        receipt
    }

    pub fn to_json(&self) -> String {
        format!(
            "{{\"schema_version\":{},\"record_type\":\"{}\",\"batch_size\":{},\"policy_hits\":{},\"policy_misses\":{},\"llm_fallbacks\":{},\"validation_passes\":{},\"validation_failures\":{},\"reuse_rate_bps\":{},\"regression_flag\":{},\"avoided_llm_calls_per_batch\":{},\"source_receipt_hash\":{},\"receipt_hash\":{}}}",
            self.schema_version,
            self.record_type,
            self.batch_size,
            self.policy_hits,
            self.policy_misses,
            self.llm_fallbacks,
            self.validation_passes,
            self.validation_failures,
            self.reuse_rate_bps,
            self.regression_flag,
            self.avoided_llm_calls_per_batch,
            self.source_receipt_hash,
            self.receipt_hash,
        )
    }

    pub fn is_valid(&self) -> bool {
        self.schema_version == 1
            && matches!(
                self.record_type,
                "policy_reuse_scale_trace"
                    | "policy_reuse_scale_trace_smoke"
                    | "policy_reuse_scale_trace_regression_smoke"
            )
            && self.batch_size > 0
            && self.policy_hits + self.policy_misses == self.batch_size
            && self.llm_fallbacks == self.policy_misses
            && self.avoided_llm_calls_per_batch == self.policy_hits
            && self.reuse_rate_bps <= 10_000
            && self.source_receipt_hash != 0
            && self.receipt_hash != 0
            && self.receipt_hash == policy_reuse_scale_trace_receipt_hash(self)
    }

    pub fn passed(&self) -> bool {
        self.is_valid() && !self.regression_flag
    }
}

impl PolicyReusePerformanceCostTrendReceipt {
    pub fn from_scale_and_cost_sources(
        scale_trace: &PolicyReuseScaleTraceReceipt,
        validation_expected_count_guarded_tests: usize,
        estimated_ms_per_guarded_test: u64,
        runtime_budget_status: &'static str,
        validation_cost_verdict: &'static str,
        source_validation_duration_hash: u64,
        source_runtime_performance_hash: u64,
    ) -> Self {
        let cost_regression_flag = scale_trace.regression_flag
            || !scale_trace.passed()
            || runtime_budget_status != "pass"
            || validation_cost_verdict != "pass";
        let mut receipt = Self {
            schema_version: 1,
            record_type: "policy_reuse_performance_cost_trend",
            batch_size: scale_trace.batch_size,
            avoided_llm_calls_per_batch: scale_trace.avoided_llm_calls_per_batch,
            reuse_rate_bps: scale_trace.reuse_rate_bps,
            validation_expected_count_guarded_tests,
            estimated_ms_per_guarded_test,
            runtime_budget_status,
            validation_cost_verdict,
            cost_regression_flag,
            source_scale_trace_hash: scale_trace.receipt_hash,
            source_validation_duration_hash,
            source_runtime_performance_hash,
            receipt_hash: 0,
        };
        receipt.receipt_hash = policy_reuse_performance_cost_trend_receipt_hash(&receipt);
        receipt
    }

    pub fn to_json(&self) -> String {
        format!(
            "{{\"schema_version\":{},\"record_type\":\"{}\",\"batch_size\":{},\"avoided_llm_calls_per_batch\":{},\"reuse_rate_bps\":{},\"validation_expected_count_guarded_tests\":{},\"estimated_ms_per_guarded_test\":{},\"runtime_budget_status\":\"{}\",\"validation_cost_verdict\":\"{}\",\"cost_regression_flag\":{},\"source_scale_trace_hash\":{},\"source_validation_duration_hash\":{},\"source_runtime_performance_hash\":{},\"receipt_hash\":{}}}",
            self.schema_version,
            self.record_type,
            self.batch_size,
            self.avoided_llm_calls_per_batch,
            self.reuse_rate_bps,
            self.validation_expected_count_guarded_tests,
            self.estimated_ms_per_guarded_test,
            self.runtime_budget_status,
            self.validation_cost_verdict,
            self.cost_regression_flag,
            self.source_scale_trace_hash,
            self.source_validation_duration_hash,
            self.source_runtime_performance_hash,
            self.receipt_hash,
        )
    }

    pub fn is_valid(&self) -> bool {
        self.schema_version == 1
            && matches!(
                self.record_type,
                "policy_reuse_performance_cost_trend"
                    | "policy_reuse_performance_cost_trend_smoke"
                    | "policy_reuse_performance_cost_trend_regression_smoke"
            )
            && self.batch_size > 0
            && self.avoided_llm_calls_per_batch <= self.batch_size
            && self.reuse_rate_bps <= 10_000
            && self.validation_expected_count_guarded_tests > 0
            && matches!(self.runtime_budget_status, "pass" | "fail")
            && matches!(self.validation_cost_verdict, "pass" | "fail")
            && self.source_scale_trace_hash != 0
            && self.source_validation_duration_hash != 0
            && self.source_runtime_performance_hash != 0
            && self.receipt_hash != 0
            && self.receipt_hash == policy_reuse_performance_cost_trend_receipt_hash(self)
    }

    pub fn passed(&self) -> bool {
        self.is_valid() && !self.cost_regression_flag
    }
}

impl PolicyReuseCostCatalogReceipt {
    #[allow(clippy::too_many_arguments)]
    pub fn from_source_hashes(
        evidence_family_count: usize,
        healthy_mode_count: usize,
        regression_mode_count: usize,
        retained_fixture_count: usize,
        required_healthy_modes_present: bool,
        required_regression_modes_present: bool,
        missing_required_modes: &'static str,
        source_policy_reuse_hash: u64,
        source_scale_trace_hash: u64,
        source_performance_cost_trend_hash: u64,
        source_validation_health_hash: u64,
        source_validation_duration_hash: u64,
        source_runtime_performance_hash: u64,
    ) -> Self {
        let summary_complete = required_healthy_modes_present
            && required_regression_modes_present
            && missing_required_modes == "none";
        let mut receipt = Self {
            schema_version: 1,
            record_type: "policy_reuse_cost_catalog",
            catalog_version: 1,
            evidence_family_count,
            healthy_mode_count,
            regression_mode_count,
            retained_fixture_count,
            required_healthy_modes_present,
            required_regression_modes_present,
            summary_complete,
            missing_required_modes,
            source_policy_reuse_hash,
            source_scale_trace_hash,
            source_performance_cost_trend_hash,
            source_validation_health_hash,
            source_validation_duration_hash,
            source_runtime_performance_hash,
            catalog_hash: 0,
            receipt_hash: 0,
        };
        receipt.catalog_hash = policy_reuse_cost_catalog_content_hash(&receipt);
        receipt.receipt_hash = policy_reuse_cost_catalog_receipt_hash(&receipt);
        receipt
    }

    pub fn to_json(&self) -> String {
        format!(
            "{{\"schema_version\":{},\"record_type\":\"{}\",\"catalog_version\":{},\"evidence_family_count\":{},\"healthy_mode_count\":{},\"regression_mode_count\":{},\"retained_fixture_count\":{},\"required_healthy_modes_present\":{},\"required_regression_modes_present\":{},\"summary_complete\":{},\"missing_required_modes\":\"{}\",\"source_policy_reuse_hash\":{},\"source_scale_trace_hash\":{},\"source_performance_cost_trend_hash\":{},\"source_validation_health_hash\":{},\"source_validation_duration_hash\":{},\"source_runtime_performance_hash\":{},\"catalog_hash\":{},\"receipt_hash\":{}}}",
            self.schema_version,
            self.record_type,
            self.catalog_version,
            self.evidence_family_count,
            self.healthy_mode_count,
            self.regression_mode_count,
            self.retained_fixture_count,
            self.required_healthy_modes_present,
            self.required_regression_modes_present,
            self.summary_complete,
            self.missing_required_modes,
            self.source_policy_reuse_hash,
            self.source_scale_trace_hash,
            self.source_performance_cost_trend_hash,
            self.source_validation_health_hash,
            self.source_validation_duration_hash,
            self.source_runtime_performance_hash,
            self.catalog_hash,
            self.receipt_hash,
        )
    }

    pub fn is_valid(&self) -> bool {
        self.schema_version == 1
            && self.catalog_version == 1
            && matches!(
                self.record_type,
                "policy_reuse_cost_catalog"
                    | "policy_reuse_cost_catalog_smoke"
                    | "policy_reuse_cost_catalog_incomplete_smoke"
            )
            && self.evidence_family_count > 0
            && self.healthy_mode_count > 0
            && self.regression_mode_count > 0
            && self.retained_fixture_count > 0
            && self.summary_complete
                == (self.required_healthy_modes_present
                    && self.required_regression_modes_present
                    && self.missing_required_modes == "none")
            && self.source_policy_reuse_hash != 0
            && self.source_scale_trace_hash != 0
            && self.source_performance_cost_trend_hash != 0
            && self.source_validation_health_hash != 0
            && self.source_validation_duration_hash != 0
            && self.source_runtime_performance_hash != 0
            && self.catalog_hash != 0
            && self.receipt_hash != 0
            && self.catalog_hash == policy_reuse_cost_catalog_content_hash(self)
            && self.receipt_hash == policy_reuse_cost_catalog_receipt_hash(self)
    }

    pub fn passed(&self) -> bool {
        self.is_valid() && self.summary_complete
    }
}

impl PolicyReuseTrendReceipt {
    pub fn to_json(&self) -> String {
        format!(
            "{{\"schema_version\":{},\"record_type\":\"{}\",\"baseline_receipt_hash\":{},\"current_receipt_hash\":{},\"baseline_retained_record_count\":{},\"current_retained_record_count\":{},\"baseline_hit_rate_bps\":{},\"current_hit_rate_bps\":{},\"hit_rate_delta_bps\":{},\"avoided_llm_call_delta\":{},\"trend_status\":\"{}\",\"verdict\":\"{}\",\"receipt_hash\":{}}}",
            self.schema_version,
            self.record_type,
            self.baseline_receipt_hash,
            self.current_receipt_hash,
            self.baseline_retained_record_count,
            self.current_retained_record_count,
            self.baseline_hit_rate_bps,
            self.current_hit_rate_bps,
            self.hit_rate_delta_bps,
            self.avoided_llm_call_delta,
            self.trend_status,
            self.verdict,
            self.receipt_hash,
        )
    }

    pub fn compare(baseline: &PolicyReuseReceipt, current: &PolicyReuseReceipt) -> Self {
        let inputs_valid = baseline.is_valid() && current.is_valid();
        let hit_rate_delta_bps = current.hit_rate_bps as i64 - baseline.hit_rate_bps as i64;
        let avoided_llm_call_delta =
            current.avoided_llm_call_count as i64 - baseline.avoided_llm_call_count as i64;
        let trend_status = if !inputs_valid {
            "invalid"
        } else if hit_rate_delta_bps >= 0 && avoided_llm_call_delta >= 0 {
            "improved_or_stable"
        } else {
            "regressed"
        };
        let verdict = if trend_status == "improved_or_stable" {
            "pass"
        } else {
            "fail"
        };
        let mut receipt = Self {
            schema_version: 1,
            record_type: "policy_reuse_trend",
            baseline_receipt_hash: baseline.receipt_hash,
            current_receipt_hash: current.receipt_hash,
            baseline_retained_record_count: baseline.retained_record_count,
            current_retained_record_count: current.retained_record_count,
            baseline_hit_rate_bps: baseline.hit_rate_bps,
            current_hit_rate_bps: current.hit_rate_bps,
            hit_rate_delta_bps,
            avoided_llm_call_delta,
            trend_status,
            verdict,
            receipt_hash: 0,
        };
        receipt.receipt_hash = policy_reuse_trend_receipt_hash(&receipt);
        receipt
    }

    pub fn is_valid(&self) -> bool {
        self.schema_version == 1
            && matches!(
                self.record_type,
                "policy_reuse_trend" | "policy_reuse_trend_smoke" | "policy_reuse_regression_smoke"
            )
            && self.baseline_receipt_hash != 0
            && self.current_receipt_hash != 0
            && self.baseline_hit_rate_bps <= 10_000
            && self.current_hit_rate_bps <= 10_000
            && self.hit_rate_delta_bps
                == self.current_hit_rate_bps as i64 - self.baseline_hit_rate_bps as i64
            && self.receipt_hash != 0
            && self.receipt_hash == policy_reuse_trend_receipt_hash(self)
    }

    pub fn passed(&self) -> bool {
        self.is_valid() && self.verdict == "pass" && self.trend_status == "improved_or_stable"
    }
}

impl PolicyReuseReceipt {
    pub fn to_json(&self) -> String {
        format!(
            "{{\"schema_version\":{},\"record_type\":\"{}\",\"retained_record_count\":{},\"policy_hit_count\":{},\"policy_miss_count\":{},\"avoided_llm_call_count\":{},\"hit_rate_bps\":{},\"record_set_hash\":{},\"verdict\":\"{}\",\"receipt_hash\":{}}}",
            self.schema_version,
            self.record_type,
            self.retained_record_count,
            self.policy_hit_count,
            self.policy_miss_count,
            self.avoided_llm_call_count,
            self.hit_rate_bps,
            self.record_set_hash,
            self.verdict,
            self.receipt_hash,
        )
    }

    pub fn from_policy_judgments(records: &[PolicyJudgmentRecord]) -> Self {
        let retained_record_count = records.len();
        let policy_hit_count = records.iter().filter(|record| record.is_valid()).count();
        let policy_miss_count = retained_record_count.saturating_sub(policy_hit_count);
        let avoided_llm_call_count = policy_hit_count;
        let hit_rate_bps = if retained_record_count == 0 {
            0
        } else {
            ((policy_hit_count as u64) * 10_000) / retained_record_count as u64
        };
        let record_set_hash = policy_reuse_record_set_hash(records);
        let verdict = if retained_record_count > 0 && policy_hit_count > 0 && record_set_hash != 0 {
            "pass"
        } else {
            "fail"
        };
        let mut receipt = Self {
            schema_version: 1,
            record_type: "policy_reuse",
            retained_record_count,
            policy_hit_count,
            policy_miss_count,
            avoided_llm_call_count,
            hit_rate_bps,
            record_set_hash,
            verdict,
            receipt_hash: 0,
        };
        receipt.receipt_hash = policy_reuse_receipt_hash(&receipt);
        receipt
    }

    pub fn is_valid(&self) -> bool {
        self.schema_version == 1
            && matches!(self.record_type, "policy_reuse" | "policy_reuse_smoke")
            && self.retained_record_count == self.policy_hit_count + self.policy_miss_count
            && self.avoided_llm_call_count == self.policy_hit_count
            && self.hit_rate_bps <= 10_000
            && self.record_set_hash != 0
            && self.receipt_hash != 0
            && self.receipt_hash == policy_reuse_receipt_hash(self)
    }

    pub fn passed(&self) -> bool {
        self.is_valid() && self.verdict == "pass"
    }
}

impl PolicyJudgmentRecord {
    pub fn from_context_policy(context: &ContextRecord, policy: &PolicyStore) -> Self {
        let (_, receipt) = policy.feedback_lookup_with_receipt();
        Self::from_context_policy_lookup_receipt(context, policy, &receipt)
    }

    pub fn from_context_policy_lookup_receipt(
        context: &ContextRecord,
        policy: &PolicyStore,
        policy_lookup_receipt: &PolicyLookupReceipt,
    ) -> Self {
        let policy_version = policy.latest_version();
        let policy_hash = policy.fingerprint();
        let lookup_valid = policy_lookup_receipt.is_valid_for(policy, POLICY_FEEDBACK_HASH);
        let policy_feedback_hash = if lookup_valid {
            policy_lookup_receipt.found_value
        } else {
            0
        };
        let policy_lookup_receipt_hash = if lookup_valid {
            policy_lookup_receipt.receipt_hash
        } else {
            0
        };
        let decision_id = policy_decision_id(
            context,
            policy_version,
            policy_hash,
            policy_feedback_hash,
            policy_lookup_receipt_hash,
        );
        let rationale_hash = policy_rationale_hash(
            context,
            decision_id,
            policy_feedback_hash,
            policy_lookup_receipt_hash,
        );
        let mut record = Self {
            context_hash: context.context_hash,
            policy_version,
            policy_hash,
            policy_feedback_hash,
            policy_lookup_receipt_hash,
            decision_id,
            rationale_hash,
            record_hash: 0,
        };
        record.record_hash = policy_judgment_record_hash(&record);
        record
    }

    pub fn decision(&self) -> PolicyJudgmentDecision {
        if self.is_valid() {
            PolicyJudgmentDecision::PolicyHit
        } else {
            PolicyJudgmentDecision::PolicyMiss
        }
    }

    pub fn is_valid(&self) -> bool {
        self.context_hash != 0
            && self.policy_version != 0
            && self.policy_hash != 0
            && self.policy_feedback_hash != 0
            && self.policy_lookup_receipt_hash != 0
            && self.decision_id != 0
            && self.rationale_hash != 0
            && self.record_hash != 0
            && self.record_hash == policy_judgment_record_hash(self)
    }

    pub fn judgment_record(&self) -> JudgmentRecord {
        if self.is_valid() {
            JudgmentRecord {
                decision_id: self.decision_id,
                policy_version: self.policy_version,
                rationale_hash: self.rationale_hash,
            }
        } else {
            JudgmentRecord {
                decision_id: 0,
                policy_version: 0,
                rationale_hash: 0,
            }
        }
    }

    pub fn submission(&self) -> EvidenceSubmission {
        self.judgment_record().submission()
    }
}

impl EvidenceProducer for PolicyJudgmentRecord {
    type Record = PolicyJudgmentRecord;

    fn record(&self) -> &Self::Record {
        self
    }

    fn submission(&self) -> EvidenceSubmission {
        PolicyJudgmentRecord::submission(self)
    }
}

fn judgment_payload_hash(record: &JudgmentRecord) -> u64 {
    let mut h = 0xaf63_dc4c_8601_ec8cu64;
    h = mix(h, record.decision_id);
    h = mix(h, record.policy_version);
    h = mix(h, record.rationale_hash);
    h.max(1)
}

fn policy_decision_id(
    context: &ContextRecord,
    policy_version: u64,
    policy_hash: u64,
    policy_feedback_hash: u64,
    policy_lookup_receipt_hash: u64,
) -> u64 {
    if !context.is_valid()
        || policy_version == 0
        || policy_hash == 0
        || policy_feedback_hash == 0
        || policy_lookup_receipt_hash == 0
    {
        return 0;
    }
    let mut h = 0x4a55_4447_504f_4c48u64;
    h = mix(h, context.objective_id);
    h = mix(h, context.context_hash);
    h = mix(h, context.memory_aggregate_hash);
    h = mix(h, policy_version);
    h = mix(h, policy_hash);
    h = mix(h, policy_feedback_hash);
    h = mix(h, policy_lookup_receipt_hash);
    h.max(1)
}

fn policy_rationale_hash(
    context: &ContextRecord,
    decision_id: u64,
    policy_feedback_hash: u64,
    policy_lookup_receipt_hash: u64,
) -> u64 {
    if !context.is_valid()
        || decision_id == 0
        || policy_feedback_hash == 0
        || policy_lookup_receipt_hash == 0
    {
        return 0;
    }
    let mut h = 0x5241_544c_504f_4c48u64;
    h = mix(h, context.observation_hash);
    h = mix(h, context.memory_receipt_hash);
    h = mix(h, decision_id);
    h = mix(h, policy_feedback_hash);
    h = mix(h, policy_lookup_receipt_hash);
    h.max(1)
}

fn policy_reuse_record_set_hash(records: &[PolicyJudgmentRecord]) -> u64 {
    if records.is_empty() {
        return 0;
    }
    let mut h = 0x504f_4c52_4555_5345u64;
    for record in records {
        h = mix(h, record.context_hash);
        h = mix(h, record.policy_version);
        h = mix(h, record.policy_hash);
        h = mix(h, record.policy_feedback_hash);
        h = mix(h, record.policy_lookup_receipt_hash);
        h = mix(h, record.decision_id);
        h = mix(h, record.rationale_hash);
        h = mix(h, record.record_hash);
        h = mix(h, u64::from(record.is_valid()));
    }
    h.max(1)
}

fn policy_reuse_trend_receipt_hash(receipt: &PolicyReuseTrendReceipt) -> u64 {
    if receipt.schema_version == 0
        || receipt.baseline_receipt_hash == 0
        || receipt.current_receipt_hash == 0
    {
        return 0;
    }
    let trend_code = match receipt.trend_status {
        "improved_or_stable" => 1,
        "regressed" => 2,
        "invalid" => 3,
        _ => 0,
    };
    let verdict_code = match receipt.verdict {
        "pass" => 1,
        "fail" => 2,
        _ => 0,
    };
    if trend_code == 0 || verdict_code == 0 {
        return 0;
    }
    let mut h = 0x504f_4c52_5452_4e44u64;
    h = mix(h, receipt.schema_version);
    h = mix(h, receipt.baseline_receipt_hash);
    h = mix(h, receipt.current_receipt_hash);
    h = mix(h, receipt.baseline_retained_record_count as u64);
    h = mix(h, receipt.current_retained_record_count as u64);
    h = mix(h, receipt.baseline_hit_rate_bps);
    h = mix(h, receipt.current_hit_rate_bps);
    h = mix(h, receipt.hit_rate_delta_bps as u64);
    h = mix(h, receipt.avoided_llm_call_delta as u64);
    h = mix(h, trend_code);
    h = mix(h, verdict_code);
    h.max(1)
}

fn policy_reuse_receipt_hash(receipt: &PolicyReuseReceipt) -> u64 {
    if receipt.schema_version == 0 || receipt.record_set_hash == 0 {
        return 0;
    }
    let verdict_code = match receipt.verdict {
        "pass" => 1,
        "fail" => 2,
        _ => 0,
    };
    if verdict_code == 0 {
        return 0;
    }
    let mut h = 0x504f_4c52_4350_5448u64;
    h = mix(h, receipt.schema_version);
    h = mix(h, receipt.retained_record_count as u64);
    h = mix(h, receipt.policy_hit_count as u64);
    h = mix(h, receipt.policy_miss_count as u64);
    h = mix(h, receipt.avoided_llm_call_count as u64);
    h = mix(h, receipt.hit_rate_bps);
    h = mix(h, receipt.record_set_hash);
    h = mix(h, verdict_code);
    h.max(1)
}

fn policy_reuse_empty_source_hash() -> u64 {
    0x504f_4c52_454d_5054u64
}

fn policy_reuse_ledger_summary_receipt_hash(receipt: &PolicyReuseLedgerSummaryReceipt) -> u64 {
    if receipt.schema_version == 0 || receipt.source_receipt_hash == 0 {
        return 0;
    }
    let record_type_code = match receipt.record_type {
        "policy_reuse_ledger_summary"
        | "policy_reuse_ledger_summary_smoke"
        | "policy_reuse_ledger_summary_regression_smoke" => 1,
        _ => 0,
    };
    if record_type_code == 0 || receipt.reuse_rate_bps > 10_000 {
        return 0;
    }
    let mut h = 0x504f_4c52_4c45_4447u64;
    h = mix(h, receipt.schema_version);
    h = mix(h, record_type_code);
    h = mix(h, receipt.policy_hits as u64);
    h = mix(h, receipt.policy_misses as u64);
    h = mix(h, receipt.llm_fallbacks as u64);
    h = mix(h, receipt.validation_passes as u64);
    h = mix(h, receipt.validation_failures as u64);
    h = mix(h, receipt.reuse_rate_bps);
    h = mix(h, u64::from(receipt.regression_flag));
    h = mix(h, receipt.source_receipt_hash);
    h.max(1)
}

fn policy_reuse_scale_trace_receipt_hash(receipt: &PolicyReuseScaleTraceReceipt) -> u64 {
    if receipt.schema_version == 0 || receipt.source_receipt_hash == 0 {
        return 0;
    }
    let record_type_code = match receipt.record_type {
        "policy_reuse_scale_trace"
        | "policy_reuse_scale_trace_smoke"
        | "policy_reuse_scale_trace_regression_smoke" => 1,
        _ => 0,
    };
    if record_type_code == 0 || receipt.reuse_rate_bps > 10_000 {
        return 0;
    }
    let mut h = 0x504f_4c52_5343_414cu64;
    h = mix(h, receipt.schema_version);
    h = mix(h, record_type_code);
    h = mix(h, receipt.batch_size as u64);
    h = mix(h, receipt.policy_hits as u64);
    h = mix(h, receipt.policy_misses as u64);
    h = mix(h, receipt.llm_fallbacks as u64);
    h = mix(h, receipt.validation_passes as u64);
    h = mix(h, receipt.validation_failures as u64);
    h = mix(h, receipt.reuse_rate_bps);
    h = mix(h, u64::from(receipt.regression_flag));
    h = mix(h, receipt.avoided_llm_calls_per_batch as u64);
    h = mix(h, receipt.source_receipt_hash);
    h.max(1)
}

fn policy_reuse_performance_cost_trend_receipt_hash(
    receipt: &PolicyReusePerformanceCostTrendReceipt,
) -> u64 {
    if receipt.schema_version == 0
        || receipt.source_scale_trace_hash == 0
        || receipt.source_validation_duration_hash == 0
        || receipt.source_runtime_performance_hash == 0
    {
        return 0;
    }
    let record_type_code = match receipt.record_type {
        "policy_reuse_performance_cost_trend"
        | "policy_reuse_performance_cost_trend_smoke"
        | "policy_reuse_performance_cost_trend_regression_smoke" => 1,
        _ => 0,
    };
    let runtime_budget_code = match receipt.runtime_budget_status {
        "pass" => 1,
        "fail" => 2,
        _ => 0,
    };
    let validation_cost_code = match receipt.validation_cost_verdict {
        "pass" => 1,
        "fail" => 2,
        _ => 0,
    };
    if record_type_code == 0
        || runtime_budget_code == 0
        || validation_cost_code == 0
        || receipt.reuse_rate_bps > 10_000
    {
        return 0;
    }
    let mut h = 0x504f_4c52_5045_5246u64;
    h = mix(h, receipt.schema_version);
    h = mix(h, record_type_code);
    h = mix(h, receipt.batch_size as u64);
    h = mix(h, receipt.avoided_llm_calls_per_batch as u64);
    h = mix(h, receipt.reuse_rate_bps);
    h = mix(h, receipt.validation_expected_count_guarded_tests as u64);
    h = mix(h, receipt.estimated_ms_per_guarded_test);
    h = mix(h, runtime_budget_code);
    h = mix(h, validation_cost_code);
    h = mix(h, u64::from(receipt.cost_regression_flag));
    h = mix(h, receipt.source_scale_trace_hash);
    h = mix(h, receipt.source_validation_duration_hash);
    h = mix(h, receipt.source_runtime_performance_hash);
    h.max(1)
}

fn policy_reuse_cost_catalog_content_hash(receipt: &PolicyReuseCostCatalogReceipt) -> u64 {
    if receipt.schema_version == 0
        || receipt.catalog_version == 0
        || receipt.source_policy_reuse_hash == 0
        || receipt.source_scale_trace_hash == 0
        || receipt.source_performance_cost_trend_hash == 0
        || receipt.source_validation_health_hash == 0
        || receipt.source_validation_duration_hash == 0
        || receipt.source_runtime_performance_hash == 0
    {
        return 0;
    }
    let record_type_code = match receipt.record_type {
        "policy_reuse_cost_catalog"
        | "policy_reuse_cost_catalog_smoke"
        | "policy_reuse_cost_catalog_incomplete_smoke" => 1,
        _ => 0,
    };
    let missing_required_modes_code = match receipt.missing_required_modes {
        "none" => 1,
        "required_regression_modes" => 2,
        "required_healthy_modes" => 3,
        "required_healthy_modes,required_regression_modes" => 4,
        _ => 0,
    };
    if record_type_code == 0 || missing_required_modes_code == 0 {
        return 0;
    }
    let mut h = 0x504f_4c52_4341_5441u64;
    h = mix(h, receipt.schema_version);
    h = mix(h, record_type_code);
    h = mix(h, receipt.catalog_version);
    h = mix(h, receipt.evidence_family_count as u64);
    h = mix(h, receipt.healthy_mode_count as u64);
    h = mix(h, receipt.regression_mode_count as u64);
    h = mix(h, receipt.retained_fixture_count as u64);
    h = mix(h, u64::from(receipt.required_healthy_modes_present));
    h = mix(h, u64::from(receipt.required_regression_modes_present));
    h = mix(h, u64::from(receipt.summary_complete));
    h = mix(h, missing_required_modes_code);
    h = mix(h, receipt.source_policy_reuse_hash);
    h = mix(h, receipt.source_scale_trace_hash);
    h = mix(h, receipt.source_performance_cost_trend_hash);
    h = mix(h, receipt.source_validation_health_hash);
    h = mix(h, receipt.source_validation_duration_hash);
    h = mix(h, receipt.source_runtime_performance_hash);
    h.max(1)
}

fn policy_reuse_cost_catalog_receipt_hash(receipt: &PolicyReuseCostCatalogReceipt) -> u64 {
    let catalog_hash = policy_reuse_cost_catalog_content_hash(receipt);
    if catalog_hash == 0 || receipt.catalog_hash != catalog_hash {
        return 0;
    }
    mix(0x504f_4c52_4341_5452u64, catalog_hash).max(1)
}

fn policy_judgment_record_hash(record: &PolicyJudgmentRecord) -> u64 {
    if record.context_hash == 0
        || record.policy_version == 0
        || record.policy_hash == 0
        || record.policy_feedback_hash == 0
        || record.policy_lookup_receipt_hash == 0
        || record.decision_id == 0
        || record.rationale_hash == 0
    {
        return 0;
    }
    let mut h = 0x504f_4c4a_5544_4754u64;
    h = mix(h, record.context_hash);
    h = mix(h, record.policy_version);
    h = mix(h, record.policy_hash);
    h = mix(h, record.policy_feedback_hash);
    h = mix(h, record.policy_lookup_receipt_hash);
    h = mix(h, record.decision_id);
    h = mix(h, record.rationale_hash);
    h.max(1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::context::ContextRecord;
    use crate::capability::learning::PolicyPromotion;
    use crate::capability::memory::{MemoryFact, MemoryIndex};
    use crate::kernel::RuntimeConfig;
    use crate::runtime::run_until_done;

    fn context() -> ContextRecord {
        let packet = crate::kernel::Packet {
            objective_id: 42,
            objective_required_tasks: 2,
            revision: 1,
            ..crate::kernel::Packet::empty()
        };
        let mut memory = MemoryIndex::default();
        assert!(memory.insert(MemoryFact::new(packet.objective_id, 0xfeed, 7, 1)));
        let (lookup, receipt) = memory.lookup_with_receipt(packet.objective_id, 8);
        ContextRecord::from_packet_memory_receipt(packet, 0xabc, &lookup, Some(&receipt))
    }

    fn promoted_policy() -> PolicyStore {
        let (_state, tlog) =
            run_until_done(crate::kernel::State::ready(), RuntimeConfig::default()).unwrap();
        let promotion = PolicyPromotion::from_tlog(&tlog, 1).unwrap();
        let mut policy = PolicyStore::default();
        policy.promote_feedback(promotion).unwrap();
        policy
    }

    #[test]
    fn policy_judgment_hit_produces_kernel_judgment_without_llm_record() {
        let context = context();
        let policy = promoted_policy();

        let record = PolicyJudgmentRecord::from_context_policy(&context, &policy);
        let judgment = record.judgment_record();
        let submission = record.submission();

        assert_eq!(record.decision(), PolicyJudgmentDecision::PolicyHit);
        assert!(record.is_valid());
        assert_eq!(record.context_hash, context.context_hash);
        assert_eq!(record.policy_hash, policy.fingerprint());
        let (_, receipt) = policy.feedback_lookup_with_receipt();
        assert_eq!(record.policy_feedback_hash, policy.feedback_hash());
        assert_eq!(record.policy_lookup_receipt_hash, receipt.receipt_hash);
        assert!(judgment.is_valid());
        assert_eq!(judgment.policy_version, policy.latest_version());
        assert_eq!(submission.gate, GateId::Judgment);
        assert_eq!(submission.evidence, Evidence::JudgmentRecord);
        assert!(submission.passed);
    }

    #[test]
    fn empty_policy_is_policy_miss_and_does_not_pass_judgment() {
        let context = context();
        let policy = PolicyStore::default();

        let record = PolicyJudgmentRecord::from_context_policy(&context, &policy);
        let submission = record.submission();

        assert_eq!(record.decision(), PolicyJudgmentDecision::PolicyMiss);
        assert!(!record.is_valid());
        assert_eq!(record.policy_feedback_hash, 0);
        assert_ne!(record.policy_lookup_receipt_hash, 0);
        assert_eq!(submission.gate, GateId::Judgment);
        assert_eq!(submission.evidence, Evidence::JudgmentRecord);
        assert!(!submission.passed);
    }

    #[test]
    fn policy_judgment_rejects_record_hash_tampering() {
        let context = context();
        let policy = promoted_policy();
        let mut record = PolicyJudgmentRecord::from_context_policy(&context, &policy);

        record.record_hash ^= 1;

        assert_eq!(record.decision(), PolicyJudgmentDecision::PolicyMiss);
        assert!(!record.is_valid());
        assert!(!record.submission().passed);
    }

    #[test]
    fn policy_judgment_rejects_tampered_policy_lookup_receipt() {
        let context = context();
        let policy = promoted_policy();
        let (_, mut receipt) = policy.feedback_lookup_with_receipt();
        receipt.found_value ^= 1;

        let record =
            PolicyJudgmentRecord::from_context_policy_lookup_receipt(&context, &policy, &receipt);

        assert_eq!(record.decision(), PolicyJudgmentDecision::PolicyMiss);
        assert_eq!(record.policy_feedback_hash, 0);
        assert_eq!(record.policy_lookup_receipt_hash, 0);
        assert!(!record.submission().passed);
    }

    #[test]
    fn policy_reuse_receipt_counts_policy_hits_as_avoided_llm_calls() {
        let context = context();
        let policy = promoted_policy();
        let hit = PolicyJudgmentRecord::from_context_policy(&context, &policy);
        let miss = PolicyJudgmentRecord::from_context_policy(&context, &PolicyStore::default());

        let receipt = PolicyReuseReceipt::from_policy_judgments(&[hit, miss]);

        assert_eq!(receipt.schema_version, 1);
        assert_eq!(receipt.record_type, "policy_reuse");
        assert_eq!(receipt.retained_record_count, 2);
        assert_eq!(receipt.policy_hit_count, 1);
        assert_eq!(receipt.policy_miss_count, 1);
        assert_eq!(receipt.avoided_llm_call_count, 1);
        assert_eq!(receipt.hit_rate_bps, 5_000);
        assert_eq!(receipt.verdict, "pass");
        assert!(receipt.passed());
    }

    #[test]
    fn policy_reuse_receipt_rejects_empty_or_tampered_aggregates() {
        let empty = PolicyReuseReceipt::from_policy_judgments(&[]);
        assert_eq!(empty.retained_record_count, 0);
        assert_eq!(empty.verdict, "fail");
        assert!(!empty.passed());

        let context = context();
        let policy = promoted_policy();
        let hit = PolicyJudgmentRecord::from_context_policy(&context, &policy);
        let mut receipt = PolicyReuseReceipt::from_policy_judgments(&[hit]);
        assert!(receipt.passed());

        receipt.avoided_llm_call_count += 1;
        assert!(!receipt.is_valid());
        assert!(!receipt.passed());
    }

    #[test]
    fn policy_reuse_trend_receipt_passes_when_policy_coverage_improves() {
        let context = context();
        let policy = promoted_policy();
        let hit = PolicyJudgmentRecord::from_context_policy(&context, &policy);
        let miss = PolicyJudgmentRecord::from_context_policy(&context, &PolicyStore::default());
        let baseline = PolicyReuseReceipt::from_policy_judgments(&[hit.clone(), miss.clone()]);
        let current = PolicyReuseReceipt::from_policy_judgments(&[hit.clone(), hit]);

        let trend = PolicyReuseTrendReceipt::compare(&baseline, &current);

        assert_eq!(trend.record_type, "policy_reuse_trend");
        assert_eq!(trend.baseline_hit_rate_bps, 5_000);
        assert_eq!(trend.current_hit_rate_bps, 10_000);
        assert_eq!(trend.hit_rate_delta_bps, 5_000);
        assert_eq!(trend.avoided_llm_call_delta, 1);
        assert_eq!(trend.trend_status, "improved_or_stable");
        assert!(trend.passed());
    }

    #[test]
    fn policy_reuse_trend_receipt_rejects_regression_or_tampering() {
        let context = context();
        let policy = promoted_policy();
        let hit = PolicyJudgmentRecord::from_context_policy(&context, &policy);
        let miss = PolicyJudgmentRecord::from_context_policy(&context, &PolicyStore::default());
        let baseline = PolicyReuseReceipt::from_policy_judgments(&[hit.clone(), hit.clone()]);
        let current = PolicyReuseReceipt::from_policy_judgments(&[hit, miss]);

        let mut trend = PolicyReuseTrendReceipt::compare(&baseline, &current);

        assert_eq!(trend.trend_status, "regressed");
        assert_eq!(trend.verdict, "fail");
        assert!(!trend.passed());
        assert!(trend.is_valid());

        trend.current_hit_rate_bps += 1;
        assert!(!trend.is_valid());
    }

    #[test]
    fn policy_reuse_ledger_summary_accepts_empty_zero_baseline() {
        let summary = PolicyReuseLedgerSummaryReceipt::empty();

        assert_eq!(summary.policy_hits, 0);
        assert_eq!(summary.policy_misses, 0);
        assert_eq!(summary.llm_fallbacks, 0);
        assert_eq!(summary.validation_passes, 0);
        assert_eq!(summary.validation_failures, 0);
        assert_eq!(summary.reuse_rate_bps, 0);
        assert!(!summary.regression_flag);
        assert!(summary.is_valid());
        assert!(summary.passed());
        assert!(summary.to_json().contains("policy_reuse_ledger_summary"));
    }

    #[test]
    fn policy_reuse_ledger_summary_counts_mixed_hit_miss_ledger() {
        let context = context();
        let policy = promoted_policy();
        let hit = PolicyJudgmentRecord::from_context_policy(&context, &policy);
        let miss = PolicyJudgmentRecord::from_context_policy(&context, &PolicyStore::default());
        let reuse = PolicyReuseReceipt::from_policy_judgments(&[hit, miss]);

        let summary = PolicyReuseLedgerSummaryReceipt::from_reuse_validation_counts(&reuse, 3, 0);

        assert_eq!(summary.policy_hits, 1);
        assert_eq!(summary.policy_misses, 1);
        assert_eq!(summary.llm_fallbacks, 1);
        assert_eq!(summary.validation_passes, 3);
        assert_eq!(summary.validation_failures, 0);
        assert_eq!(summary.reuse_rate_bps, 5_000);
        assert_eq!(summary.source_receipt_hash, reuse.receipt_hash);
        assert!(!summary.regression_flag);
        assert!(summary.passed());
    }

    #[test]
    fn policy_reuse_ledger_summary_flags_validation_regression() {
        let context = context();
        let policy = promoted_policy();
        let hit = PolicyJudgmentRecord::from_context_policy(&context, &policy);
        let reuse = PolicyReuseReceipt::from_policy_judgments(&[hit]);

        let summary = PolicyReuseLedgerSummaryReceipt::from_reuse_validation_counts(&reuse, 2, 1);

        assert_eq!(summary.policy_hits, 1);
        assert_eq!(summary.policy_misses, 0);
        assert_eq!(summary.llm_fallbacks, 0);
        assert_eq!(summary.validation_passes, 2);
        assert_eq!(summary.validation_failures, 1);
        assert_eq!(summary.reuse_rate_bps, 10_000);
        assert!(summary.regression_flag);
        assert!(summary.is_valid());
        assert!(!summary.passed());
    }

    #[test]
    fn policy_reuse_ledger_summary_rejects_tampered_counts_or_sequence_binding() {
        let context = context();
        let policy = promoted_policy();
        let hit = PolicyJudgmentRecord::from_context_policy(&context, &policy);
        let reuse = PolicyReuseReceipt::from_policy_judgments(&[hit]);
        let mut summary =
            PolicyReuseLedgerSummaryReceipt::from_reuse_validation_counts(&reuse, 1, 0);
        assert!(summary.passed());

        summary.llm_fallbacks += 1;
        assert!(!summary.is_valid());

        let mut rebound =
            PolicyReuseLedgerSummaryReceipt::from_reuse_validation_counts(&reuse, 1, 0);
        rebound.source_receipt_hash ^= 1;
        assert!(!rebound.is_valid());
    }

    #[test]
    fn policy_reuse_scale_trace_counts_larger_batch_avoided_calls() {
        let context = context();
        let policy = promoted_policy();
        let hit = PolicyJudgmentRecord::from_context_policy(&context, &policy);
        let miss = PolicyJudgmentRecord::from_context_policy(&context, &PolicyStore::default());
        let reuse = PolicyReuseReceipt::from_policy_judgments(&[
            hit.clone(),
            hit.clone(),
            hit.clone(),
            hit,
            miss.clone(),
            miss,
        ]);

        let trace = PolicyReuseScaleTraceReceipt::from_reuse_validation_counts(&reuse, 6, 6, 0);

        assert_eq!(trace.record_type, "policy_reuse_scale_trace");
        assert_eq!(trace.batch_size, 6);
        assert_eq!(trace.policy_hits, 4);
        assert_eq!(trace.policy_misses, 2);
        assert_eq!(trace.llm_fallbacks, 2);
        assert_eq!(trace.avoided_llm_calls_per_batch, 4);
        assert_eq!(trace.reuse_rate_bps, 6_666);
        assert!(!trace.regression_flag);
        assert!(trace.passed());
    }

    #[test]
    fn policy_reuse_scale_trace_rejects_regression_or_tampering() {
        let context = context();
        let policy = promoted_policy();
        let hit = PolicyJudgmentRecord::from_context_policy(&context, &policy);
        let miss = PolicyJudgmentRecord::from_context_policy(&context, &PolicyStore::default());
        let reuse = PolicyReuseReceipt::from_policy_judgments(&[hit.clone(), hit, miss]);

        let mut trace = PolicyReuseScaleTraceReceipt::from_reuse_validation_counts(&reuse, 3, 2, 1);

        assert!(trace.regression_flag);
        assert!(trace.is_valid());
        assert!(!trace.passed());

        trace.avoided_llm_calls_per_batch += 1;
        assert!(!trace.is_valid());
    }

    #[test]
    fn policy_reuse_performance_cost_trend_binds_scale_and_cost_sources() {
        let context = context();
        let policy = promoted_policy();
        let hit = PolicyJudgmentRecord::from_context_policy(&context, &policy);
        let miss = PolicyJudgmentRecord::from_context_policy(&context, &PolicyStore::default());
        let reuse = PolicyReuseReceipt::from_policy_judgments(&[
            hit.clone(),
            hit.clone(),
            hit.clone(),
            hit,
            miss.clone(),
            miss,
        ]);
        let scale = PolicyReuseScaleTraceReceipt::from_reuse_validation_counts(&reuse, 6, 6, 0);

        let mut receipt = PolicyReusePerformanceCostTrendReceipt::from_scale_and_cost_sources(
            &scale, 146, 20, "pass", "pass", 0x101, 0x202,
        );

        assert_eq!(receipt.record_type, "policy_reuse_performance_cost_trend");
        assert_eq!(receipt.batch_size, 6);
        assert_eq!(receipt.avoided_llm_calls_per_batch, 4);
        assert_eq!(receipt.reuse_rate_bps, 6_666);
        assert_eq!(receipt.validation_expected_count_guarded_tests, 146);
        assert_eq!(receipt.estimated_ms_per_guarded_test, 20);
        assert_eq!(receipt.runtime_budget_status, "pass");
        assert_eq!(receipt.validation_cost_verdict, "pass");
        assert!(!receipt.cost_regression_flag);
        assert_eq!(receipt.source_scale_trace_hash, scale.receipt_hash);
        assert!(receipt.passed());

        receipt.source_runtime_performance_hash ^= 1;
        assert!(!receipt.is_valid());
    }

    #[test]
    fn policy_reuse_performance_cost_trend_flags_cost_regressions() {
        let context = context();
        let policy = promoted_policy();
        let hit = PolicyJudgmentRecord::from_context_policy(&context, &policy);
        let miss = PolicyJudgmentRecord::from_context_policy(&context, &PolicyStore::default());
        let reuse = PolicyReuseReceipt::from_policy_judgments(&[hit.clone(), hit, miss]);
        let scale = PolicyReuseScaleTraceReceipt::from_reuse_validation_counts(&reuse, 3, 3, 0);

        let receipt = PolicyReusePerformanceCostTrendReceipt::from_scale_and_cost_sources(
            &scale, 146, 22, "pass", "fail", 0x303, 0x404,
        );

        assert!(receipt.cost_regression_flag);
        assert!(receipt.is_valid());
        assert!(!receipt.passed());
    }

    #[test]
    fn policy_reuse_cost_catalog_binds_required_source_hashes() {
        let receipt = PolicyReuseCostCatalogReceipt::from_source_hashes(
            6, 4, 4, 6, true, true, "none", 0x101, 0x202, 0x303, 0x404, 0x505, 0x606,
        );

        assert_eq!(receipt.record_type, "policy_reuse_cost_catalog");
        assert_eq!(receipt.catalog_version, 1);
        assert_eq!(receipt.evidence_family_count, 6);
        assert_eq!(receipt.healthy_mode_count, 4);
        assert_eq!(receipt.regression_mode_count, 4);
        assert_eq!(receipt.retained_fixture_count, 6);
        assert!(receipt.required_healthy_modes_present);
        assert!(receipt.required_regression_modes_present);
        assert!(receipt.summary_complete);
        assert_eq!(receipt.missing_required_modes, "none");
        assert_ne!(receipt.catalog_hash, 0);
        assert!(receipt.passed());

        let mut tampered = receipt.clone();
        tampered.source_runtime_performance_hash ^= 1;
        assert!(!tampered.is_valid());
    }

    #[test]
    fn policy_reuse_cost_catalog_flags_incomplete_coverage() {
        let receipt = PolicyReuseCostCatalogReceipt::from_source_hashes(
            6,
            4,
            3,
            6,
            true,
            false,
            "required_regression_modes",
            0x101,
            0x202,
            0x303,
            0x404,
            0x505,
            0x606,
        );

        assert!(receipt.is_valid());
        assert!(!receipt.summary_complete);
        assert!(!receipt.required_regression_modes_present);
        assert_eq!(receipt.missing_required_modes, "required_regression_modes");
        assert!(!receipt.passed());
    }
}
