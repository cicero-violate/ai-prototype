//! Pure domain record contracts.
//!
//! These records are descriptor-only. They do not mutate runtime state, append
//! TLog events, or bypass capability verification.

use serde::{Deserialize, Serialize};

pub const DOMAIN_SCHEMA_VERSION: &str = "canon_domain_v1";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DomainSchemaVersion {
    V1,
}

impl DomainSchemaVersion {
    pub const CURRENT: Self = Self::V1;

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::V1 => DOMAIN_SCHEMA_VERSION,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DomainId {
    Unknown,
    GlobalIntelligence,
    Finance,
    Business,
    TradingSandbox,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DomainSourceKind {
    Unknown,
    UserObjective,
    WorldSignal,
    MarketData,
    BusinessSystem,
    CustomerFeedback,
    RuntimeTrace,
    GraphEvidence,
    EvaluationReceipt,
    PolicyStore,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DomainVerdict {
    Ignore,
    Watch,
    Research,
    ActBusiness,
    ActFinanceResearch,
    SimulateTrading,
    Block,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DomainBridgeTarget {
    ObservationRecord,
    ContextRecord,
    JudgmentRecord,
    PlanRecord,
    VerificationRecord,
    EvalRecord,
    PolicyPromotion,
    Blocked,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DomainHorizon {
    Immediate,
    Tactical,
    Strategic,
    Secular,
}

pub type Horizon = DomainHorizon;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DomainSignalClass {
    Unknown,
    MacroTrend,
    Regulation,
    TechnologyShift,
    CustomerNeed,
    BusinessWorkflow,
    FinanceResearch,
    TradingSimulation,
    RiskAlert,
    CapabilityLearning,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DomainRiskClass {
    Unknown,
    Low,
    Medium,
    High,
    Critical,
    Blocked,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DomainPlanKind {
    ResearchPlan,
    BusinessWorkflowPlan,
    FinanceAnalysisPlan,
    FinanceResearch,
    AllocationHypothesisPlan,
    TradingSimulationPlan,
    TradingSimulation,
    LearningPromotionPlan,
}

pub type PlanKind = DomainPlanKind;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum DomainLiveEffectLevel {
    None,
    ReadOnly,
    SandboxWrite,
    ExternalWrite,
    FinancialExecution,
}

pub type LiveEffectLevel = DomainLiveEffectLevel;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DomainContractError {
    EmptyField(&'static str),
    ScoreOutOfRange { field: &'static str, value: u16 },
    UnsafeLiveEffect { field: &'static str },
}

fn require_non_empty(field: &'static str, value: &str) -> Result<(), DomainContractError> {
    if value.trim().is_empty() {
        Err(DomainContractError::EmptyField(field))
    } else {
        Ok(())
    }
}

fn require_hash(field: &'static str, value: &str) -> Result<(), DomainContractError> {
    require_non_empty(field, value)
}

fn require_score(field: &'static str, value: u16) -> Result<u16, DomainContractError> {
    if value <= 1000 {
        Ok(value)
    } else {
        Err(DomainContractError::ScoreOutOfRange { field, value })
    }
}

fn require_not_financial_execution(
    field: &'static str,
    value: DomainLiveEffectLevel,
) -> Result<DomainLiveEffectLevel, DomainContractError> {
    if value == DomainLiveEffectLevel::FinancialExecution {
        Err(DomainContractError::UnsafeLiveEffect { field })
    } else {
        Ok(value)
    }
}

fn require_plan_live_effect(
    plan_kind: PlanKind,
    requested_live_effect_level: LiveEffectLevel,
) -> Result<LiveEffectLevel, DomainContractError> {
    match plan_kind {
        PlanKind::FinanceResearch | PlanKind::FinanceAnalysisPlan
            if requested_live_effect_level > LiveEffectLevel::ReadOnly =>
        {
            Err(DomainContractError::UnsafeLiveEffect {
                field: "requested_live_effect_level",
            })
        }
        PlanKind::TradingSimulation | PlanKind::TradingSimulationPlan
            if requested_live_effect_level > LiveEffectLevel::SandboxWrite =>
        {
            Err(DomainContractError::UnsafeLiveEffect {
                field: "requested_live_effect_level",
            })
        }
        _ => require_not_financial_execution(
            "requested_live_effect_level",
            requested_live_effect_level,
        ),
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DomainSignal {
    pub schema_version: &'static str,
    pub signal_id: String,
    pub domain_id: DomainId,
    pub source_id: String,
    pub observed_at: String,
    pub horizon: Horizon,
    pub signal_class: DomainSignalClass,
    pub payload_hash: String,
    pub provenance_hash: String,
    pub source_quality_score: u16,
    pub freshness_score: u16,
    pub contradiction_score: u16,
}

impl DomainSignal {
    pub fn new(
        signal_id: impl Into<String>,
        domain_id: DomainId,
        source_id: impl Into<String>,
        observed_at: impl Into<String>,
        horizon: Horizon,
        signal_class: DomainSignalClass,
        payload_hash: impl Into<String>,
        provenance_hash: impl Into<String>,
        source_quality_score: u16,
        freshness_score: u16,
        contradiction_score: u16,
    ) -> Result<Self, DomainContractError> {
        let signal_id = signal_id.into();
        let source_id = source_id.into();
        let observed_at = observed_at.into();
        let payload_hash = payload_hash.into();
        let provenance_hash = provenance_hash.into();

        require_non_empty("signal_id", &signal_id)?;
        require_non_empty("source_id", &source_id)?;
        require_non_empty("observed_at", &observed_at)?;
        require_hash("payload_hash", &payload_hash)?;
        require_hash("provenance_hash", &provenance_hash)?;

        Ok(Self {
            schema_version: DOMAIN_SCHEMA_VERSION,
            signal_id,
            domain_id,
            source_id,
            observed_at,
            horizon,
            signal_class,
            payload_hash,
            provenance_hash,
            source_quality_score: require_score("source_quality_score", source_quality_score)?,
            freshness_score: require_score("freshness_score", freshness_score)?,
            contradiction_score: require_score("contradiction_score", contradiction_score)?,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DomainContext {
    pub schema_version: &'static str,
    pub context_id: String,
    pub domain_id: DomainId,
    pub signal_hash: String,
    pub context_hash: String,
    pub provenance_hash: String,
    pub horizon: Horizon,
    pub source_kind: DomainSourceKind,
    pub context_quality_score: u16,
    pub staleness_score: u16,
}

impl DomainContext {
    pub fn new(
        context_id: impl Into<String>,
        domain_id: DomainId,
        signal_hash: impl Into<String>,
        context_hash: impl Into<String>,
        provenance_hash: impl Into<String>,
        horizon: Horizon,
        source_kind: DomainSourceKind,
        context_quality_score: u16,
        staleness_score: u16,
    ) -> Result<Self, DomainContractError> {
        let context_id = context_id.into();
        let signal_hash = signal_hash.into();
        let context_hash = context_hash.into();
        let provenance_hash = provenance_hash.into();

        require_non_empty("context_id", &context_id)?;
        require_hash("signal_hash", &signal_hash)?;
        require_hash("context_hash", &context_hash)?;
        require_hash("provenance_hash", &provenance_hash)?;

        Ok(Self {
            schema_version: DOMAIN_SCHEMA_VERSION,
            context_id,
            domain_id,
            signal_hash,
            context_hash,
            provenance_hash,
            horizon,
            source_kind,
            context_quality_score: require_score("context_quality_score", context_quality_score)?,
            staleness_score: require_score("staleness_score", staleness_score)?,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DomainRiskEnvelope {
    pub schema_version: &'static str,
    pub envelope_id: String,
    pub domain_id: DomainId,
    pub max_uncertainty: u16,
    pub min_confidence: u16,
    pub max_staleness: u16,
    pub max_live_effect_level: LiveEffectLevel,
    pub requires_human_review: bool,
    pub requires_verification: bool,
    pub sandbox_only: bool,
}

impl DomainRiskEnvelope {
    pub fn new(
        envelope_id: impl Into<String>,
        domain_id: DomainId,
        max_uncertainty: u16,
        min_confidence: u16,
        max_staleness: u16,
        max_live_effect_level: LiveEffectLevel,
        requires_human_review: bool,
        requires_verification: bool,
        sandbox_only: bool,
    ) -> Result<Self, DomainContractError> {
        let envelope_id = envelope_id.into();
        require_non_empty("envelope_id", &envelope_id)?;

        Ok(Self {
            schema_version: DOMAIN_SCHEMA_VERSION,
            envelope_id,
            domain_id,
            max_uncertainty: require_score("max_uncertainty", max_uncertainty)?,
            min_confidence: require_score("min_confidence", min_confidence)?,
            max_staleness: require_score("max_staleness", max_staleness)?,
            max_live_effect_level: require_not_financial_execution(
                "max_live_effect_level",
                max_live_effect_level,
            )?,
            requires_human_review,
            requires_verification,
            sandbox_only,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DomainJudgment {
    pub schema_version: &'static str,
    pub judgment_id: String,
    pub domain_id: DomainId,
    pub opportunity_score: u16,
    pub risk_score: u16,
    pub confidence_score: u16,
    pub uncertainty_score: u16,
    pub actionability_score: u16,
    pub policy_fit_score: u16,
    pub expected_value_score: u16,
    pub verdict: DomainVerdict,
    pub rationale_hash: String,
}

impl DomainJudgment {
    pub fn new(
        judgment_id: impl Into<String>,
        domain_id: DomainId,
        opportunity_score: u16,
        risk_score: u16,
        confidence_score: u16,
        uncertainty_score: u16,
        actionability_score: u16,
        policy_fit_score: u16,
        expected_value_score: u16,
        verdict: DomainVerdict,
        rationale_hash: impl Into<String>,
    ) -> Result<Self, DomainContractError> {
        let judgment_id = judgment_id.into();
        let rationale_hash = rationale_hash.into();

        require_non_empty("judgment_id", &judgment_id)?;
        require_hash("rationale_hash", &rationale_hash)?;

        Ok(Self {
            schema_version: DOMAIN_SCHEMA_VERSION,
            judgment_id,
            domain_id,
            opportunity_score: require_score("opportunity_score", opportunity_score)?,
            risk_score: require_score("risk_score", risk_score)?,
            confidence_score: require_score("confidence_score", confidence_score)?,
            uncertainty_score: require_score("uncertainty_score", uncertainty_score)?,
            actionability_score: require_score("actionability_score", actionability_score)?,
            policy_fit_score: require_score("policy_fit_score", policy_fit_score)?,
            expected_value_score: require_score("expected_value_score", expected_value_score)?,
            verdict,
            rationale_hash,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DomainPlan {
    pub schema_version: &'static str,
    pub plan_id: String,
    pub domain_id: DomainId,
    pub judgment_hash: String,
    pub plan_kind: PlanKind,
    pub bridge_target: DomainBridgeTarget,
    pub required_capability_set_hash: String,
    pub expected_receipt_set_hash: String,
    pub risk_envelope_hash: String,
    pub success_metric_hash: String,
    pub rollback_or_invalidation_hash: String,
    pub requested_live_effect_level: LiveEffectLevel,
}

impl DomainPlan {
    pub fn new(
        plan_id: impl Into<String>,
        domain_id: DomainId,
        judgment_hash: impl Into<String>,
        plan_kind: PlanKind,
        bridge_target: DomainBridgeTarget,
        required_capability_set_hash: impl Into<String>,
        expected_receipt_set_hash: impl Into<String>,
        risk_envelope_hash: impl Into<String>,
        success_metric_hash: impl Into<String>,
        rollback_or_invalidation_hash: impl Into<String>,
        requested_live_effect_level: LiveEffectLevel,
    ) -> Result<Self, DomainContractError> {
        let plan_id = plan_id.into();
        let judgment_hash = judgment_hash.into();
        let required_capability_set_hash = required_capability_set_hash.into();
        let expected_receipt_set_hash = expected_receipt_set_hash.into();
        let risk_envelope_hash = risk_envelope_hash.into();
        let success_metric_hash = success_metric_hash.into();
        let rollback_or_invalidation_hash = rollback_or_invalidation_hash.into();

        require_non_empty("plan_id", &plan_id)?;
        require_hash("judgment_hash", &judgment_hash)?;
        require_hash(
            "required_capability_set_hash",
            &required_capability_set_hash,
        )?;
        require_hash("expected_receipt_set_hash", &expected_receipt_set_hash)?;
        require_hash("risk_envelope_hash", &risk_envelope_hash)?;
        require_hash("success_metric_hash", &success_metric_hash)?;
        require_hash(
            "rollback_or_invalidation_hash",
            &rollback_or_invalidation_hash,
        )?;
        let requested_live_effect_level = require_plan_live_effect(
            plan_kind,
            requested_live_effect_level,
        )?;

        Ok(Self {
            schema_version: DOMAIN_SCHEMA_VERSION,
            plan_id,
            domain_id,
            judgment_hash,
            plan_kind,
            bridge_target,
            required_capability_set_hash,
            expected_receipt_set_hash,
            risk_envelope_hash,
            success_metric_hash,
            rollback_or_invalidation_hash,
            requested_live_effect_level,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DomainEval {
    pub schema_version: &'static str,
    pub eval_id: String,
    pub domain_id: DomainId,
    pub plan_hash: String,
    pub result_hash: String,
    pub correctness_score: u16,
    pub usefulness_score: u16,
    pub risk_adherence_score: u16,
    pub roi_or_value_score: u16,
    pub reproducibility_score: u16,
    pub promotion_allowed: bool,
}

impl DomainEval {
    pub fn new(
        eval_id: impl Into<String>,
        domain_id: DomainId,
        plan_hash: impl Into<String>,
        result_hash: impl Into<String>,
        correctness_score: u16,
        usefulness_score: u16,
        risk_adherence_score: u16,
        roi_or_value_score: u16,
        reproducibility_score: u16,
        promotion_allowed: bool,
    ) -> Result<Self, DomainContractError> {
        let eval_id = eval_id.into();
        let plan_hash = plan_hash.into();
        let result_hash = result_hash.into();

        require_non_empty("eval_id", &eval_id)?;
        require_hash("plan_hash", &plan_hash)?;
        require_hash("result_hash", &result_hash)?;

        Ok(Self {
            schema_version: DOMAIN_SCHEMA_VERSION,
            eval_id,
            domain_id,
            plan_hash,
            result_hash,
            correctness_score: require_score("correctness_score", correctness_score)?,
            usefulness_score: require_score("usefulness_score", usefulness_score)?,
            risk_adherence_score: require_score("risk_adherence_score", risk_adherence_score)?,
            roi_or_value_score: require_score("roi_or_value_score", roi_or_value_score)?,
            reproducibility_score: require_score("reproducibility_score", reproducibility_score)?,
            promotion_allowed,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DomainPromotionCandidate {
    pub schema_version: &'static str,
    pub candidate_id: String,
    pub domain_id: DomainId,
    pub source_eval_set_hash: String,
    pub repeated_pattern_hash: String,
    pub policy_delta_hash: String,
    pub expected_gain_score: u16,
    pub regression_risk_score: u16,
    pub promotion_verdict: DomainVerdict,
}

impl DomainPromotionCandidate {
    pub fn new(
        candidate_id: impl Into<String>,
        domain_id: DomainId,
        source_eval_set_hash: impl Into<String>,
        repeated_pattern_hash: impl Into<String>,
        policy_delta_hash: impl Into<String>,
        expected_gain_score: u16,
        regression_risk_score: u16,
        promotion_verdict: DomainVerdict,
    ) -> Result<Self, DomainContractError> {
        let candidate_id = candidate_id.into();
        let source_eval_set_hash = source_eval_set_hash.into();
        let repeated_pattern_hash = repeated_pattern_hash.into();
        let policy_delta_hash = policy_delta_hash.into();

        require_non_empty("candidate_id", &candidate_id)?;
        require_hash("source_eval_set_hash", &source_eval_set_hash)?;
        require_hash("repeated_pattern_hash", &repeated_pattern_hash)?;
        require_hash("policy_delta_hash", &policy_delta_hash)?;

        Ok(Self {
            schema_version: DOMAIN_SCHEMA_VERSION,
            candidate_id,
            domain_id,
            source_eval_set_hash,
            repeated_pattern_hash,
            policy_delta_hash,
            expected_gain_score: require_score("expected_gain_score", expected_gain_score)?,
            regression_risk_score: require_score("regression_risk_score", regression_risk_score)?,
            promotion_verdict,
        })
    }
}

impl Default for DomainRiskEnvelope {
    fn default() -> Self {
        Self {
            schema_version: DOMAIN_SCHEMA_VERSION,
            envelope_id: String::new(),
            domain_id: DomainId::Unknown,
            max_uncertainty: 1000,
            min_confidence: 0,
            max_staleness: 1000,
            max_live_effect_level: LiveEffectLevel::None,
            requires_human_review: false,
            requires_verification: true,
            sandbox_only: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn domain_schema_primitives_round_trip_through_json() {
        let version = DomainSchemaVersion::CURRENT;
        let encoded = serde_json::to_string(&version).expect("schema version serializes");
        let decoded: DomainSchemaVersion =
            serde_json::from_str(&encoded).expect("schema version deserializes");
        assert_eq!(decoded, DomainSchemaVersion::V1);
        assert_eq!(decoded.as_str(), DOMAIN_SCHEMA_VERSION);

        let primitives = serde_json::json!({
            "source_kind": DomainSourceKind::GraphEvidence,
            "horizon": DomainHorizon::Strategic,
            "signal_class": DomainSignalClass::BusinessWorkflow,
            "risk_class": DomainRiskClass::High,
            "plan_kind": DomainPlanKind::BusinessWorkflowPlan,
            "live_effect_level": DomainLiveEffectLevel::SandboxWrite
        });

        assert_eq!(primitives["source_kind"], "GraphEvidence");
        assert_eq!(primitives["horizon"], "Strategic");
        assert_eq!(primitives["signal_class"], "BusinessWorkflow");
        assert_eq!(primitives["risk_class"], "High");
        assert_eq!(primitives["plan_kind"], "BusinessWorkflowPlan");
        assert_eq!(primitives["live_effect_level"], "SandboxWrite");
    }

    #[test]
    fn live_effect_order_preserves_safety_boundary() {
        assert!(DomainLiveEffectLevel::None < DomainLiveEffectLevel::ReadOnly);
        assert!(DomainLiveEffectLevel::ReadOnly < DomainLiveEffectLevel::SandboxWrite);
        assert!(DomainLiveEffectLevel::SandboxWrite < DomainLiveEffectLevel::ExternalWrite);
        assert!(DomainLiveEffectLevel::ExternalWrite < DomainLiveEffectLevel::FinancialExecution);
    }

    #[test]
    fn risk_envelope_constructor_rejects_financial_execution() {
        let result = DomainRiskEnvelope::new(
            "env-1",
            DomainId::TradingSandbox,
            500,
            400,
            300,
            DomainLiveEffectLevel::FinancialExecution,
            true,
            true,
            true,
        );

        assert_eq!(
            result,
            Err(DomainContractError::UnsafeLiveEffect {
                field: "max_live_effect_level"
            })
        );
    }

    #[test]
    fn domain_record_constructors_validate_invariants() {
        let envelope = DomainRiskEnvelope::new(
            "env-success",
            DomainId::Business,
            250,
            700,
            300,
            DomainLiveEffectLevel::SandboxWrite,
            false,
            true,
            true,
        )
        .expect("risk envelope constructor accepts bounded safe inputs");
        assert_eq!(envelope.schema_version, DOMAIN_SCHEMA_VERSION);
        assert_eq!(envelope.envelope_id, "env-success");
        assert_eq!(envelope.max_uncertainty, 250);
        assert_eq!(envelope.min_confidence, 700);
        assert_eq!(
            envelope.max_live_effect_level,
            DomainLiveEffectLevel::SandboxWrite
        );
        assert!(envelope.requires_verification);
        assert!(envelope.sandbox_only);

        let judgment = DomainJudgment::new(
            "judgment-success",
            DomainId::Business,
            800,
            200,
            850,
            150,
            750,
            900,
            700,
            DomainVerdict::ActBusiness,
            "hash:rationale",
        )
        .expect("judgment constructor accepts bounded scores and rationale hash");
        assert_eq!(judgment.schema_version, DOMAIN_SCHEMA_VERSION);
        assert_eq!(judgment.judgment_id, "judgment-success");
        assert_eq!(judgment.domain_id, DomainId::Business);
        assert_eq!(judgment.opportunity_score, 800);
        assert_eq!(judgment.verdict, DomainVerdict::ActBusiness);
        assert_eq!(judgment.rationale_hash, "hash:rationale");

        let plan = DomainPlan::new(
            "plan-success",
            DomainId::Business,
            "hash:judgment",
            DomainPlanKind::BusinessWorkflowPlan,
            DomainBridgeTarget::PlanRecord,
            "hash:capability-set",
            "hash:receipt-set",
            "hash:risk-envelope",
            "hash:success-metric",
            "hash:rollback-or-invalidation",
            DomainLiveEffectLevel::SandboxWrite,
        )
        .expect("plan constructor accepts hashes and safe live effect");
        assert_eq!(plan.schema_version, DOMAIN_SCHEMA_VERSION);
        assert_eq!(plan.plan_id, "plan-success");
        assert_eq!(plan.judgment_hash, "hash:judgment");
        assert_eq!(plan.plan_kind, DomainPlanKind::BusinessWorkflowPlan);
        assert_eq!(plan.bridge_target, DomainBridgeTarget::PlanRecord);
        assert_eq!(
            plan.requested_live_effect_level,
            DomainLiveEffectLevel::SandboxWrite
        );

        let eval = DomainEval::new(
            "eval-success",
            DomainId::Business,
            "hash:plan",
            "hash:result",
            950,
            850,
            900,
            700,
            800,
            true,
        )
        .expect("eval constructor accepts hashes and bounded scores");
        assert_eq!(eval.schema_version, DOMAIN_SCHEMA_VERSION);
        assert_eq!(eval.eval_id, "eval-success");
        assert_eq!(eval.plan_hash, "hash:plan");
        assert_eq!(eval.result_hash, "hash:result");
        assert_eq!(eval.correctness_score, 950);
        assert!(eval.promotion_allowed);

        let candidate = DomainPromotionCandidate::new(
            "candidate-success",
            DomainId::Business,
            "hash:eval-set",
            "hash:pattern",
            "hash:policy-delta",
            875,
            125,
            DomainVerdict::ActBusiness,
        )
        .expect("promotion candidate constructor accepts hashes and bounded scores");
        assert_eq!(candidate.schema_version, DOMAIN_SCHEMA_VERSION);
        assert_eq!(candidate.candidate_id, "candidate-success");
        assert_eq!(candidate.source_eval_set_hash, "hash:eval-set");
        assert_eq!(candidate.expected_gain_score, 875);
        assert_eq!(candidate.regression_risk_score, 125);
        assert_eq!(candidate.promotion_verdict, DomainVerdict::ActBusiness);
    }
}
