//! Domain architecture sketch.
//!
//! This module sketches the pure domain layer that should sit above raw world
//! inputs and below existing capability evidence production.
//!
//! Boundary:
//! - domain: finance/global-intelligence/business/trading interpretation
//! - capability: evidence records, receipts, tools, verification
//! - kernel: frozen state machine
//! - runtime: deterministic transitions and replay
//!
//! Non-goals:
//! - no I/O, process spawning, or network access
//! - no direct `State` mutation
//! - no direct `TLog` append
//! - no command-ledger mutation
//! - no changes to `kernel`
//! - no changes to `runtime::reduce`
//! - no live trading path
//! - no I/O, process, network, runtime, command-ledger, or TLog mutation authority

pub mod bridge;
pub mod business;
pub mod contracts;
pub mod finance;
pub mod global_intelligence;
pub mod identity;
pub mod risk;
pub mod scoring;
pub mod trading;

pub use bridge::{bridge_target_for_verdict, default_plan_kind};
pub use business::{
    monetization_score, BusinessOpportunity, CustomerFeedbackSignal, WorkflowAutomationCandidate,
    WorkflowId,
};
pub use contracts::{
    DomainBridgeTarget, DomainEval, DomainHorizon, DomainId, DomainJudgment, DomainLiveEffectLevel,
    DomainPlan, DomainPlanKind, DomainPromotionCandidate, DomainRiskClass, DomainRiskEnvelope,
    DomainSchemaVersion, DomainSignal, DomainSignalClass, DomainSourceKind, DomainVerdict, Horizon,
    LiveEffectLevel, PlanKind, DOMAIN_SCHEMA_VERSION,
};
pub use finance::{
    finance_research_allowed, AssetUniverse, FinanceHypothesis, FinanceRiskDimensions,
};
pub use global_intelligence::{
    actionability_hint, stale_for_horizon, GlobalSignalProfile, SignalClass,
};
pub use identity::stable_domain_id;
pub use risk::{evaluate_risk_envelope, DomainRiskDecision};
pub use scoring::{
    actionability_score, bounded_product, domain_value_score, DomainScoreInputs, Score,
};
pub use trading::{
    enforce_sandbox_only, BacktestReceiptRequirements, TradingRiskLimit, TradingSimulationPlan,
};
