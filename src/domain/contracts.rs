//! Pure domain record contracts.
//!
//! These records are descriptor-only. They do not mutate runtime state, append
//! TLog events, or bypass capability verification.

pub const DOMAIN_SCHEMA_VERSION: &str = "canon_domain_v1";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DomainId {
    Unknown,
    GlobalIntelligence,
    Finance,
    Business,
    TradingSandbox,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DomainVerdict {
    Ignore,
    Watch,
    Research,
    ActBusiness,
    ActFinanceResearch,
    SimulateTrading,
    Block,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Horizon {
    Immediate,
    Tactical,
    Strategic,
    Secular,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlanKind {
    ResearchPlan,
    BusinessWorkflowPlan,
    FinanceAnalysisPlan,
    AllocationHypothesisPlan,
    TradingSimulationPlan,
    LearningPromotionPlan,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum LiveEffectLevel {
    None,
    ReadOnly,
    SandboxWrite,
    ExternalWrite,
    FinancialExecution,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DomainSignal {
    pub schema_version: &'static str,
    pub signal_id: String,
    pub domain_id: DomainId,
    pub source_id: String,
    pub observed_at: String,
    pub horizon: Horizon,
    pub signal_class: String,
    pub payload_hash: String,
    pub provenance_hash: String,
    pub source_quality_score: u16,
    pub freshness_score: u16,
    pub contradiction_score: u16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
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

#[derive(Clone, Debug, PartialEq, Eq)]
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DomainPlan {
    pub schema_version: &'static str,
    pub plan_id: String,
    pub domain_id: DomainId,
    pub judgment_hash: String,
    pub plan_kind: PlanKind,
    pub bridge_target: DomainBridgeTarget,
    pub risk_envelope_hash: String,
    pub success_metric_hash: String,
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
